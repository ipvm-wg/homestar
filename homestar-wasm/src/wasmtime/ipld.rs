//! Convert (bidirectionally) between [Ipld] and [wasmtime::component::Val]s
//! and [wasmtime::component::Type]s.
//!
//! tl;dr: [Ipld] <=> [wasmtime::component::Val] IR.
//!
//! Export restrictions to be aware of!:
//! <https://github.com/bytecodealliance/wasm-tools/blob/main/tests/local/component-model/type-export-restrictions.wast>
//!
//! [Ipld]: libipld::Ipld

use crate::error::{InterpreterError, TagsError};
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use itertools::{
    FoldWhile::{Continue, Done},
    Itertools, Position,
};
use libipld::{
    cid::{
        self,
        multibase::{self, Base},
        Cid,
    },
    Ipld,
};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use std::{
    collections::{BTreeMap, VecDeque},
    iter,
    rc::Rc,
    str,
};
use wasmtime::component::{Type, Val};

/// Interface-type wrapper over wasmtime component [wasmtime::component::Type].
#[derive(Clone, Debug, Default)]
pub enum InterfaceType<'a> {
    /// Works for `any` [wasmtime::component::Type].
    #[default]
    Any,
    /// Wraps [wasmtime::component::Type].
    Type(Type),
    /// Wraps [wasmtime::component::Type] ref.
    TypeRef(&'a Type),
}

impl InterfaceType<'_> {
    #[allow(dead_code)]
    fn into_inner(self) -> Option<Type> {
        match self {
            InterfaceType::Type(ty) => Some(ty),
            InterfaceType::TypeRef(ty) => Some(ty.to_owned()),
            _ => None,
        }
    }

    fn inner(&self) -> Option<&Type> {
        match self {
            InterfaceType::Type(ty) => Some(ty),
            InterfaceType::TypeRef(ty) => Some(ty),
            _ => None,
        }
    }
}

impl<'a> From<&'a Type> for InterfaceType<'a> {
    fn from(typ: &'a Type) -> Self {
        match typ {
            Type::List(_)
            | Type::Record(_)
            | Type::Union(_)
            | Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Float32
            | Type::Float64 => InterfaceType::TypeRef(typ),
            _ => InterfaceType::Any,
        }
    }
}

/// Shared [AtomicRefCell] for adding/popping `named` tags on a
/// referenced stack.
#[derive(Debug, PartialEq)]
pub struct Tags(Rc<AtomicRefCell<VecDeque<String>>>);

impl Tags {
    fn new(tags: VecDeque<String>) -> Self {
        Tags(Rc::new(AtomicRefCell::new(tags)))
    }

    #[allow(dead_code)]
    fn borrow_mut(&self) -> AtomicRefMut<'_, VecDeque<String>> {
        self.0.borrow_mut()
    }

    fn try_borrow_mut(&self) -> Result<AtomicRefMut<'_, VecDeque<String>>, TagsError> {
        Ok(self.0.try_borrow_mut()?)
    }

    fn borrow(&self) -> AtomicRef<'_, VecDeque<String>> {
        self.0.borrow()
    }

    #[allow(dead_code)]
    fn try_borrow(&self) -> Result<AtomicRef<'_, VecDeque<String>>, TagsError> {
        Ok(self.0.try_borrow()?)
    }

    fn push(&mut self, tag: String) -> Result<(), TagsError> {
        self.try_borrow_mut()?.push_front(tag);
        Ok(())
    }

    fn pop(&self) -> Result<String, TagsError> {
        self.try_borrow_mut()?
            .pop_front()
            .ok_or(TagsError::TagsEmpty)
    }

    fn empty(&self) -> bool {
        self.borrow().is_empty()
    }
}

impl Clone for Tags {
    fn clone(&self) -> Self {
        Tags(Rc::clone(&self.0))
    }
}

impl Default for Tags {
    fn default() -> Self {
        Tags::new(VecDeque::new())
    }
}

/// Wrapper-type for runtime [wasmtime::component::Val].
#[derive(Debug, PartialEq)]
pub struct RuntimeVal(Val, Tags);

impl RuntimeVal {
    /// Return [wasmtime::component::Val] from [RuntimeVal] wrapper.
    pub fn into_inner(self) -> (Val, Tags) {
        (self.0, self.1)
    }

    /// Return [wasmtime::component::Val] from [RuntimeVal] wrapper.
    pub fn value(self) -> Val {
        self.0
    }

    /// Instantiate [RuntimeVal] type without a tag.
    pub fn new(val: Val) -> Self {
        RuntimeVal(val, Tags::default())
    }

    /// Instantiate [RuntimeVal] type with a tag.
    pub fn new_with_tags(val: Val, tags: Tags) -> Self {
        RuntimeVal(val, tags)
    }

    /// Convert from [Ipld] to [RuntimeVal] with a given [InterfaceType].
    ///
    /// TODOs:
    ///  * check Unions over lists and tags
    ///  * Enums
    ///  * Structs / Records
    ///  * Results / Options
    pub fn try_from(
        ipld: Ipld,
        interface_ty: &InterfaceType<'_>,
    ) -> Result<Self, InterpreterError> {
        // TODO: Configure for recursion.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            let dyn_type = match ipld {
                Ipld::Map(mut v)
                    if matches!(interface_ty.inner(), Some(Type::Union(_))) && v.len() == 1 =>
                {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<union>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    // already pattern matched against
                    let union_inst = inner.unwrap_union();

                    let (key, elem) = v.pop_first().ok_or_else(|| {
                        InterpreterError::MapType(
                            "IPLD map must contain at least one discriminant".to_string(),
                        )
                    })?;

                    let (discriminant, mut tags) = union_inst
                        .types()
                        .zip(iter::repeat(elem))
                        .enumerate()
                        .with_position()
                        .fold_while(Ok((Ok(Val::Bool(false)), Tags::default())), |acc, pos| {
                            let is_last = matches!(pos.0, Position::Last | Position::Only);
                            let (idx, (ty, elem)) = pos.1;
                            match RuntimeVal::try_from(elem, &InterfaceType::TypeRef(&ty)) {
                                Ok(RuntimeVal(value, tags)) => {
                                    if value.ty() == ty {
                                        Done(Ok((union_inst.new_val(idx as u32, value), tags)))
                                    } else {
                                        Continue(acc)
                                    }
                                }
                                Err(err) => {
                                    if is_last {
                                        Done(Err(InterpreterError::NoDiscriminantMatched(
                                            err.to_string(),
                                        )))
                                    } else {
                                        Continue(acc)
                                    }
                                }
                            }
                        })
                        .into_inner()?;

                    tags.push(key)?;

                    RuntimeVal::new_with_tags(discriminant?, tags)
                }
                v if matches!(interface_ty.inner(), Some(Type::Union(_))) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<union>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    // already pattern matched against
                    let union_inst = inner.unwrap_union();

                    let discriminant = union_inst
                        .types()
                        .zip(iter::repeat(v))
                        .enumerate()
                        .with_position()
                        .fold_while(Ok(Val::Bool(false)), |acc, pos| {
                            let is_last = matches!(pos.0, Position::Last | Position::Only);
                            let (idx, (ty, elem)) = pos.1;
                            match RuntimeVal::try_from(elem, &InterfaceType::TypeRef(&ty)) {
                                Ok(RuntimeVal(value, _tags)) => {
                                    if value.ty() == ty {
                                        Done(union_inst.new_val(idx as u32, value))
                                    } else {
                                        Continue(acc)
                                    }
                                }
                                Err(err) => {
                                    if is_last {
                                        Done(Err(InterpreterError::NoDiscriminantMatched(
                                            err.to_string(),
                                        )
                                        .into()))
                                    } else {
                                        Continue(acc)
                                    }
                                }
                            }
                        })
                        .into_inner()?;

                    RuntimeVal::new(discriminant)
                }
                Ipld::Null => match interface_ty {
                    InterfaceType::Type(Type::String)
                    | InterfaceType::TypeRef(Type::String)
                    | InterfaceType::Any => RuntimeVal::new(Val::String(Box::from("null"))),
                    _ => Err(InterpreterError::WitToIpld(Ipld::Null))?,
                },
                Ipld::Bool(v) => match interface_ty {
                    InterfaceType::Type(Type::Bool)
                    | InterfaceType::TypeRef(Type::Bool)
                    | InterfaceType::Any => RuntimeVal::new(Val::Bool(v)),
                    _ => Err(InterpreterError::WitToIpld(Ipld::Bool(v)))?,
                },
                Ipld::Integer(v) => match interface_ty {
                    InterfaceType::Type(Type::U8) | InterfaceType::TypeRef(Type::U8) => {
                        RuntimeVal::new(Val::U8(v.try_into()?))
                    }
                    InterfaceType::Type(Type::U16) | InterfaceType::TypeRef(Type::U16) => {
                        RuntimeVal::new(Val::U16(v.try_into()?))
                    }
                    InterfaceType::Type(Type::U32) | InterfaceType::TypeRef(Type::U32) => {
                        RuntimeVal::new(Val::U32(v.try_into()?))
                    }
                    InterfaceType::Type(Type::U64) | InterfaceType::TypeRef(Type::U64) => {
                        RuntimeVal::new(Val::U64(v.try_into()?))
                    }
                    InterfaceType::Type(Type::S8) | InterfaceType::TypeRef(Type::S8) => {
                        RuntimeVal::new(Val::S8(v.try_into()?))
                    }
                    InterfaceType::Type(Type::S16) | InterfaceType::TypeRef(Type::S16) => {
                        RuntimeVal::new(Val::S16(v.try_into()?))
                    }
                    InterfaceType::Type(Type::S32) | InterfaceType::TypeRef(Type::S32) => {
                        RuntimeVal::new(Val::S32(v.try_into()?))
                    }
                    InterfaceType::Any
                    | InterfaceType::Type(Type::S64)
                    | InterfaceType::TypeRef(Type::S64) => RuntimeVal::new(Val::S64(v.try_into()?)),
                    _ => Err(InterpreterError::WitToIpld(Ipld::Integer(v)))?,
                },
                Ipld::Float(v) => match interface_ty {
                    InterfaceType::Type(Type::Float32) | InterfaceType::TypeRef(Type::Float32) => {
                        RuntimeVal::new(Val::Float32(v as f32))
                    }
                    _ => RuntimeVal::new(Val::Float64(v)),
                },
                Ipld::String(v) => RuntimeVal::new(Val::String(Box::from(v))),
                Ipld::Bytes(v) if matches!(interface_ty.inner(), Some(Type::List(_))) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<list<u8>>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    // already pattern matched against
                    let list_inst = inner.unwrap_list();

                    let vec = v.into_iter().fold(vec![], |mut acc, elem| {
                        let RuntimeVal(value, _tags) = RuntimeVal::new(Val::U8(elem));
                        acc.push(value);
                        acc
                    });

                    RuntimeVal::new(list_inst.new_val(vec.into_boxed_slice())?)
                }
                Ipld::Bytes(v) => RuntimeVal::new(Val::String(Box::from(Base::Base64.encode(v)))),
                Ipld::Link(v) => match v.version() {
                    cid::Version::V0 => RuntimeVal::new(Val::String(Box::from(
                        v.to_string_of_base(Base::Base58Btc)?,
                    ))),
                    cid::Version::V1 => RuntimeVal::new(Val::String(Box::from(
                        v.to_string_of_base(Base::Base32Lower)?,
                    ))),
                },
                Ipld::List(v) if matches!(interface_ty.inner(), Some(Type::List(_))) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<list>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    // already pattern matched against
                    let list_inst = inner.unwrap_list();

                    let vec = v.into_iter().try_fold(vec![], |mut acc, elem| {
                        let RuntimeVal(value, _tags) =
                            RuntimeVal::try_from(elem, &InterfaceType::Type(list_inst.ty()))?;
                        acc.push(value);
                        Ok::<_, InterpreterError>(acc)
                    })?;

                    RuntimeVal::new(list_inst.new_val(vec.into_boxed_slice())?)
                }
                // Handle edge-casing via Ipld representations.
                // i.e. true, as [true] as part of a Ipld-schema union.
                Ipld::List(v) => v
                    .into_iter()
                    .fold_while(Ok(RuntimeVal::new(Val::Bool(false))), |_acc, elem| {
                        match RuntimeVal::try_from(elem, interface_ty) {
                            Ok(runtime_val) => Done(Ok(runtime_val)),
                            Err(e) => Done(Err(e)),
                        }
                    })
                    .into_inner()?,

                Ipld::Map(v) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<List>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    let list_inst = matches!(inner, Type::List(_))
                        .then_some(inner.unwrap_list())
                        .ok_or_else(|| InterpreterError::TypeMismatch {
                            expected: "<list>".to_string(),
                            given: Some(format!("{inner:#?}")),
                        })?;

                    let tuple_inst = matches!(list_inst.ty(), Type::Tuple(_))
                        .then_some(list_inst.ty().unwrap_tuple())
                        .ok_or_else(|| InterpreterError::TypeMismatch {
                            expected: "<list>".to_string(),
                            given: Some(format!("{inner:#?}")),
                        })?
                        .to_owned();

                    let ty = tuple_inst.types().nth(1).ok_or_else(|| {
                        InterpreterError::MapType(
                            "IPLD map must have tuples of two elements".to_string(),
                        )
                    })?;

                    let (vec, tags) = v.into_iter().try_fold(
                        (vec![], VecDeque::new()),
                        |(mut acc_tuples, mut acc_tags), (key, elem)| {
                            let RuntimeVal(value, tags) =
                                RuntimeVal::try_from(elem, &InterfaceType::TypeRef(&ty))?;

                            let tuple = Box::new([Val::String(Box::from(key)), value]);
                            let new_tuple = tuple_inst.new_val(tuple)?;
                            acc_tuples.push(new_tuple);
                            let mut tags = tags.try_borrow_mut()?;
                            (acc_tags).append(&mut tags);
                            Ok::<_, InterpreterError>((acc_tuples, acc_tags))
                        },
                    )?;
                    RuntimeVal::new_with_tags(
                        list_inst.new_val(vec.into_boxed_slice())?,
                        Tags::new(tags),
                    )
                }
            };

            Ok(dyn_type)
        })
    }
}

impl TryFrom<RuntimeVal> for Ipld {
    type Error = InterpreterError;

    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        fn base_64_bytes(s: &str) -> Result<Vec<u8>, multibase::Error> {
            Base::Base64.decode(s)
        }
        fn cid(s: &str) -> Result<Cid, cid::Error> {
            Cid::try_from(s)
        }
        // TODO: Configure for recursion.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            let ipld = match val {
                RuntimeVal(Val::Char(c), _) => Ipld::String(c.to_string()),
                RuntimeVal(Val::String(v), _) => match v.to_string() {
                    s if s.eq("null") => Ipld::Null,
                    s => {
                        if let Ok(cid) = cid(&s) {
                            Ipld::Link(cid)
                        } else if let Ok(decoded) = base_64_bytes(&s) {
                            Ipld::Bytes(decoded)
                        } else {
                            Ipld::String(s)
                        }
                    }
                },
                RuntimeVal(Val::Bool(v), _) => Ipld::Bool(v),
                RuntimeVal(Val::U8(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::U16(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::U32(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::U64(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::S8(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::S16(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::S32(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::S64(v), _) => Ipld::Integer(v.into()),
                RuntimeVal(Val::Float32(v), _) => {
                    // Convert to decimal for handling precision issues going from
                    // f32 => f64.
                    let dec =
                        Decimal::from_f32(v).ok_or_else(|| InterpreterError::FloatToDecimal(v))?;
                    Ipld::Float(
                        dec.to_f64()
                            .ok_or_else(|| InterpreterError::DecimalToFloat(dec))?,
                    )
                }
                RuntimeVal(Val::Float64(v), _) => Ipld::Float(v),
                RuntimeVal(Val::List(v), tags) if matches!(v.ty().ty(), Type::Tuple(_)) => {
                    let inner = v.iter().try_fold(BTreeMap::new(), |mut acc, elem| {
                        if let Val::Tuple(tup) = elem {
                            let tup_values = tup.values();
                            if let [Val::String(s), v] = tup_values {
                                let ipld = Ipld::try_from(RuntimeVal::new_with_tags(
                                    v.to_owned(),
                                    tags.clone(),
                                ))?;
                                acc.insert(s.to_string(), ipld);
                                Ok::<_, Self::Error>(acc)
                            } else {
                                Err(InterpreterError::TypeMismatch {
                                    expected: "<tuple> of (<string>, <&wasmtime::Val>)".to_string(),
                                    given: Some(format!("{tup_values:#?}")),
                                })?
                            }
                        } else {
                            Err(InterpreterError::TypeMismatch {
                                expected: "<tuple>".to_string(),
                                given: Some(format!("{elem:#?}")),
                            })?
                        }
                    })?;
                    Ipld::Map(inner)
                }
                RuntimeVal(Val::List(v), _) => match v.first() {
                    Some(Val::U8(_)) => {
                        let inner = v.iter().try_fold(vec![], |mut acc, elem| {
                            if let Val::U8(v) = elem {
                                acc.push(v.to_owned());
                                Ok::<_, Self::Error>(acc)
                            } else {
                                Err(InterpreterError::TypeMismatch {
                                    expected: "all <u8> types".to_string(),
                                    given: Some(format!("{elem:#?}")),
                                })?
                            }
                        })?;

                        Ipld::Bytes(inner)
                    }
                    Some(_) => {
                        let inner = v.iter().try_fold(vec![], |mut acc, elem| {
                            let ipld = Ipld::try_from(RuntimeVal::new(elem.to_owned()))?;
                            acc.push(ipld);
                            Ok::<_, Self::Error>(acc)
                        })?;
                        Ipld::List(inner)
                    }
                    None => Ipld::List(vec![]),
                },
                RuntimeVal(Val::Union(u), tags) if !tags.empty() => {
                    let inner = Ipld::try_from(RuntimeVal::new(u.payload().to_owned()))?;

                    // Keep tag.
                    let tag = tags.pop()?;
                    Ipld::from(BTreeMap::from([(tag, inner)]))
                }
                RuntimeVal(Val::Union(u), tags) if tags.empty() => {
                    Ipld::try_from(RuntimeVal::new(u.payload().to_owned()))?
                }
                // Rest of Wit types are unhandled going to Ipld.
                v => Err(InterpreterError::IpldToWit(format!("{v:#?}")))?,
            };

            Ok(ipld)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;
    use libipld::{
        cbor::DagCborCodec,
        ipld,
        multihash::{Code, MultihashDigest},
        prelude::Encode,
        DagCbor,
    };
    use serde_ipld_dagcbor::from_slice;
    use std::collections::BTreeMap;

    const RAW: u64 = 0x55;

    #[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
    #[ipld(repr = "keyed")]
    enum KeyedUnion {
        #[ipld(repr = "value")]
        A(bool),
        #[ipld(rename = "b")]
        #[ipld(repr = "value")]
        B(u16),
        #[ipld(repr = "value")]
        C(u16),
    }

    #[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
    #[ipld(repr = "kinded")]
    enum KindedUnion {
        #[ipld(repr = "value")]
        A(bool),
        #[ipld(rename = "b")]
        #[ipld(repr = "value")]
        B(u16),
        #[ipld(repr = "value")]
        C(u16),
    }

    #[test]
    fn try_null_roundtrip() {
        let runtime_null = RuntimeVal::new(Val::String(Box::from("null")));

        assert_eq!(
            RuntimeVal::try_from(Ipld::Null, &InterfaceType::Any).unwrap(),
            runtime_null
        );

        assert_eq!(Ipld::try_from(runtime_null).unwrap(), Ipld::Null);
    }

    #[test]
    fn try_bool_roundtrip() {
        let runtime_bool = RuntimeVal::new(Val::Bool(false));

        assert_eq!(
            RuntimeVal::try_from(Ipld::Bool(false), &InterfaceType::Any).unwrap(),
            runtime_bool
        );

        assert_eq!(Ipld::try_from(runtime_bool).unwrap(), Ipld::Bool(false));
    }

    #[test]
    fn try_integer_unsignedu8_type_roundtrip() {
        let ipld = Ipld::Integer(8);
        let runtime_int = RuntimeVal::new(Val::U8(8));

        let ty = test_utils::component::setup_component("u8".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_unsigned16_type_roundtrip() {
        let ipld = Ipld::Integer(8829);
        let runtime_int = RuntimeVal::new(Val::U16(8829));

        let ty = test_utils::component::setup_component("u16".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_unsigned32_type_roundtrip() {
        let ipld = Ipld::Integer(8829);
        let runtime_int = RuntimeVal::new(Val::U32(8829));

        let ty = test_utils::component::setup_component("u32".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_unsigned64_type_roundtrip() {
        let ipld = Ipld::Integer(8829);
        let runtime_int = RuntimeVal::new(Val::U64(8829));

        let ty = test_utils::component::setup_component_with_param(
            "u64".to_string(),
            &[test_utils::component::Param(
                test_utils::component::Type::I64,
                Some(0),
            )],
        );

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_any_roundtrip() {
        let ipld = Ipld::Integer(2828829);
        let runtime_int = RuntimeVal::new(Val::S64(2828829));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_signedu8_type_roundtrip() {
        let ipld = Ipld::Integer(1);
        let runtime_int = RuntimeVal::new(Val::S8(1));

        let ty = test_utils::component::setup_component("s8".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_signed16_type_roundtrip() {
        let ipld = Ipld::Integer(-8829);
        let runtime_int = RuntimeVal::new(Val::S16(-8829));

        let ty = test_utils::component::setup_component("s16".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_signed32_type_roundtrip() {
        let ipld = Ipld::Integer(-8829);
        let runtime_int = RuntimeVal::new(Val::S32(-8829));

        let ty = test_utils::component::setup_component("s32".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_signed64_type_roundtrip() {
        let ipld = Ipld::Integer(-8829);
        let runtime_int = RuntimeVal::new(Val::S64(-8829));

        let ty = test_utils::component::setup_component_with_param(
            "s64".to_string(),
            &[test_utils::component::Param(
                test_utils::component::Type::I64,
                Some(0),
            )],
        );
        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_float_any_roundtrip() {
        let ipld = Ipld::Float(3883.20);
        let runtime_float = RuntimeVal::new(Val::Float64(3883.20));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime_float
        );

        assert_eq!(Ipld::try_from(runtime_float).unwrap(), ipld);
    }

    #[test]
    fn try_float_type_roundtrip() {
        let ipld = Ipld::Float(3883.20);
        let runtime_float = RuntimeVal::new(Val::Float32(3883.20));

        let ty = test_utils::component::setup_component_with_param(
            "float32".to_string(),
            &[test_utils::component::Param(
                test_utils::component::Type::F32,
                Some(0),
            )],
        );

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_float
        );

        assert_eq!(Ipld::try_from(runtime_float).unwrap(), ipld);
    }

    #[test]
    fn try_string_roundtrip() {
        let ipld = Ipld::String("Hello!".into());
        let runtime = RuntimeVal::new(Val::String(Box::from("Hello!")));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_bytes_roundtrip() {
        let bytes = b"hell0".to_vec();
        let ipld = Ipld::Bytes(bytes.clone());
        let encoded_cid = Base::Base64.encode(bytes);
        let runtime = RuntimeVal::new(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_cid_v1_roundtrip() {
        let h = Code::Blake3_256.digest(b"beep boop");
        let cid = Cid::new_v1(RAW, h);
        let ipld = Ipld::Link(cid);
        let encoded_cid = cid.to_string_of_base(Base::Base32Lower).unwrap();
        let runtime = RuntimeVal::new(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_cid_v0_roundtrip() {
        let h = Code::Sha2_256.digest(b"beep boop");
        let cid = Cid::new_v0(h).unwrap();
        let ipld = Ipld::Link(cid);
        let encoded_cid = cid.to_string_of_base(Base::Base58Btc).unwrap();
        let runtime = RuntimeVal::new(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_list_roundtrip() {
        let ipld = Ipld::List(vec![Ipld::Integer(22), Ipld::Integer(32)]);

        let ty = test_utils::component::setup_component("(list s64)".to_string(), 8);

        let val_list = ty
            .unwrap_list()
            .new_val(Box::new([Val::S64(22), Val::S64(32)]))
            .unwrap();

        let runtime = RuntimeVal::new(val_list);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_list_of_list_roundtrip() {
        let ipld = Ipld::List(vec![
            Ipld::List(vec![Ipld::String("a".to_string())]),
            Ipld::List(vec![Ipld::String("b".to_string())]),
        ]);

        let ty = test_utils::component::setup_component("(list (list string))".to_string(), 8);

        let inner_list = ty.unwrap_list();

        let first_list = inner_list
            .ty()
            .unwrap_list()
            .new_val(Box::new([Val::String(Box::from("a"))]))
            .unwrap();

        let second_list = inner_list
            .ty()
            .unwrap_list()
            .new_val(Box::new([Val::String(Box::from("b"))]))
            .unwrap();

        let val_list = ty
            .unwrap_list()
            .new_val(Box::new([first_list, second_list]))
            .unwrap();

        let runtime = RuntimeVal::new(val_list);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_union_roundtrip_keyed() {
        let mut bytes_kind_b = Vec::new();
        KeyedUnion::B(2)
            .encode(DagCborCodec, &mut bytes_kind_b)
            .unwrap();
        let ipld_kind_b: Ipld = from_slice(&bytes_kind_b).unwrap();

        let ty = test_utils::component::setup_component("(union bool u16)".to_string(), 8);
        let unwrapped = ty.unwrap_union();

        let val_union_1 = unwrapped.new_val(1, Val::U16(2)).unwrap();
        let runtime_1 =
            RuntimeVal::new_with_tags(val_union_1, Tags::new(vec!["b".to_string()].into()));

        assert_eq!(
            RuntimeVal::try_from(ipld_kind_b.clone(), &InterfaceType::Type(ty.clone())).unwrap(),
            runtime_1
        );

        assert_eq!(Ipld::try_from(runtime_1).unwrap(), ipld_kind_b);

        let val_union_0 = unwrapped.new_val(0, Val::Bool(true)).unwrap();
        let runtime_0 =
            RuntimeVal::new_with_tags(val_union_0, Tags::new(vec!["A".to_string()].into()));

        let mut bytes_kind_a = Vec::new();
        KeyedUnion::A(true)
            .encode(DagCborCodec, &mut bytes_kind_a)
            .unwrap();
        let ipld_kind_a: Ipld = from_slice(&bytes_kind_a).unwrap();

        assert_eq!(
            RuntimeVal::try_from(ipld_kind_a.clone(), &InterfaceType::Type(ty.clone())).unwrap(),
            runtime_0
        );

        assert_eq!(Ipld::try_from(runtime_0).unwrap(), ipld_kind_a);
    }

    #[test]
    fn try_union_roundtrip_kinded() {
        let mut bytes_kind_b = Vec::new();
        KindedUnion::B(2)
            .encode(DagCborCodec, &mut bytes_kind_b)
            .unwrap();
        let ipld_kind_b: Ipld = from_slice(&bytes_kind_b).unwrap();

        let ty = test_utils::component::setup_component("(union bool u16)".to_string(), 8);
        let unwrapped = ty.unwrap_union();

        let val_union_1 = unwrapped.new_val(1, Val::U16(2)).unwrap();
        let runtime_1 = RuntimeVal::new(val_union_1);

        assert_eq!(
            RuntimeVal::try_from(ipld_kind_b.clone(), &InterfaceType::Type(ty.clone())).unwrap(),
            runtime_1
        );

        assert_eq!(Ipld::try_from(runtime_1).unwrap(), ipld_kind_b);

        let val_union_0 = unwrapped.new_val(0, Val::Bool(true)).unwrap();
        let runtime_0 = RuntimeVal::new(val_union_0);

        let mut bytes_kind_a = Vec::new();
        KindedUnion::A(true)
            .encode(DagCborCodec, &mut bytes_kind_a)
            .unwrap();
        let ipld_kind_a: Ipld = from_slice(&bytes_kind_a).unwrap();

        assert_eq!(
            RuntimeVal::try_from(ipld_kind_a.clone(), &InterfaceType::Type(ty.clone())).unwrap(),
            runtime_0
        );

        assert_eq!(Ipld::try_from(runtime_0).unwrap(), ipld_kind_a);
    }

    #[test]
    fn try_map_roundtrip() {
        let ipld = Ipld::Map(BTreeMap::from([
            ("test".into(), Ipld::String("Hello!".into())),
            ("test1".into(), Ipld::String("Hello!".into())),
        ]));

        let ty =
            test_utils::component::setup_component("(list (tuple string string))".to_string(), 8);

        let tuple1 = [
            Val::String(Box::from("test")),
            Val::String(Box::from("Hello!")),
        ];

        let tuple2 = [
            Val::String(Box::from("test1")),
            Val::String(Box::from("Hello!")),
        ];

        let unwrapped = ty.unwrap_list();

        let val_tuple1 = unwrapped
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple1))
            .unwrap();
        let val_tuple2 = unwrapped
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple2))
            .unwrap();

        let val_map = unwrapped
            .new_val(Box::new([val_tuple1, val_tuple2]))
            .unwrap();

        let runtime = RuntimeVal::new(val_map);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_list_with_nested_map_roundtrip() {
        let ipld_map1 = Ipld::Map(BTreeMap::from([
            ("test".into(), Ipld::String("Hello!".into())),
            ("test1".into(), Ipld::String("Hello!".into())),
        ]));

        let ipld_map2 = Ipld::Map(BTreeMap::from([
            ("test2".into(), Ipld::String("Hello!".into())),
            ("test3".into(), Ipld::String("Hello!".into())),
        ]));

        let ipld = Ipld::List(vec![ipld_map1.clone(), ipld_map2.clone()]);

        let ty = test_utils::component::setup_component(
            "(list (list (tuple string string)))".to_string(),
            8,
        );

        let tuple1 = [
            Val::String(Box::from("test")),
            Val::String(Box::from("Hello!")),
        ];

        let tuple2 = [
            Val::String(Box::from("test1")),
            Val::String(Box::from("Hello!")),
        ];

        let tuple3 = [
            Val::String(Box::from("test2")),
            Val::String(Box::from("Hello!")),
        ];

        let tuple4 = [
            Val::String(Box::from("test3")),
            Val::String(Box::from("Hello!")),
        ];

        let unwrapped_outer_list = ty.unwrap_list();

        let first_inner_tuple = unwrapped_outer_list
            .ty()
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple1))
            .unwrap();

        let second_inner_tuple = unwrapped_outer_list
            .ty()
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple2))
            .unwrap();

        let third_inner_tuple = unwrapped_outer_list
            .ty()
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple3))
            .unwrap();

        let fourth_inner_tuple = unwrapped_outer_list
            .ty()
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple4))
            .unwrap();

        let first_inner_list = ty
            .unwrap_list()
            .ty()
            .unwrap_list()
            .new_val(Box::new([first_inner_tuple, second_inner_tuple]))
            .unwrap();

        let second_inner_list = ty
            .unwrap_list()
            .ty()
            .unwrap_list()
            .new_val(Box::new([third_inner_tuple, fourth_inner_tuple]))
            .unwrap();

        let val_outer_list = ty
            .unwrap_list()
            .new_val(Box::new([first_inner_list, second_inner_list]))
            .unwrap();

        let runtime = RuntimeVal::new(val_outer_list);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime
        );

        //assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }
}
