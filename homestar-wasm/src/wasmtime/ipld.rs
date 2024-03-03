//! Convert (bidirectionally) between Ipld and [wasmtime::component::Val]s
//! and [wasmtime::component::Type]s.
//!
//! tl;dr: Ipld <=> [wasmtime::component::Val] IR.
//!
//! Export restrictions to be aware of!:
//! <https://github.com/bytecodealliance/wasm-tools/blob/main/tests/local/component-model/type-export-restrictions.wast>

use crate::error::{InterpreterError, TagsError};
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use homestar_invocation::ensure;
use indexmap::IndexMap;
use itertools::{FoldWhile::Done, Itertools};
use libipld::{
    cid::{self, multibase::Base, Cid},
    Ipld,
};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use std::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
    str,
};
use wasmtime::component::{Type, Val};

const DEFAULT_RED_ZONE: usize = 32 * 1024;
const DEFAULT_EXTRA_STACK: usize = 1024 * 1024;

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
            | Type::Tuple(_)
            | Type::Variant(_)
            | Type::Option(_)
            | Type::Result(_)
            | Type::Flags(_)
            | Type::Enum(_)
            | Type::String
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

    #[allow(dead_code)]
    fn push(&mut self, tag: String) -> Result<(), TagsError> {
        self.try_borrow_mut()?.push_front(tag);
        Ok(())
    }

    #[allow(dead_code)]
    fn pop(&self) -> Result<String, TagsError> {
        self.try_borrow_mut()?
            .pop_front()
            .ok_or(TagsError::TagsEmpty)
    }

    #[allow(dead_code)]
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

    /// Convert from Ipld to [RuntimeVal] with a given [InterfaceType].
    pub fn try_from(
        ipld: Ipld,
        interface_ty: &InterfaceType<'_>,
    ) -> Result<Self, InterpreterError> {
        // TODO: Configure for recursion.
        stacker::maybe_grow(DEFAULT_RED_ZONE, DEFAULT_EXTRA_STACK, || {
            let dyn_type = match ipld {
                Ipld::Null => match interface_ty {
                    InterfaceType::Type(Type::Option(opt_inst)) => {
                        RuntimeVal::new(opt_inst.new_val(None)?)
                    }
                    InterfaceType::Type(Type::String)
                    | InterfaceType::TypeRef(Type::String)
                    | InterfaceType::Any => RuntimeVal::new(Val::String(Box::from("null"))),
                    _ => Err(InterpreterError::IpldToWit(
                        "No conversion possible".to_string(),
                    ))?,
                },
                v if matches!(interface_ty.inner(), Some(Type::Option(_))) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<option>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    let opt_inst = inner.unwrap_option();
                    let inner_v = RuntimeVal::try_from(v, &InterfaceType::TypeRef(&opt_inst.ty()))?;
                    RuntimeVal::new(opt_inst.new_val(Some(inner_v.value()))?)
                }
                v if matches!(interface_ty.inner(), Some(Type::Result(_))) => {
                    let inner =
                        interface_ty
                            .inner()
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<result>".to_string(),
                                given: interface_ty.inner().map(|t| format!("{t:#?}")),
                            })?;

                    let res_inst = inner.unwrap_result();
                    let ok_typ = res_inst.ok();
                    let err_typ = res_inst.err();

                    if let Ipld::List(vec) = v {
                        ensure!(
                            vec.len() == 2,
                            InterpreterError::IpldToWit(
                                "IPLD map (as WIT result) as must have two elements".to_string(),
                            )
                        );
                        match (vec.as_slice(), ok_typ, err_typ) {
                            ([ipld, Ipld::Null], Some(ty), _) => {
                                let inner_v = RuntimeVal::try_from(
                                    ipld.to_owned(),
                                    &InterfaceType::TypeRef(&ty),
                                )?;
                                RuntimeVal::new(res_inst.new_val(Ok(Some(inner_v.value())))?)
                            }
                            ([Ipld::Null, ipld], _, Some(ty)) => {
                                let inner_v = RuntimeVal::try_from(
                                    ipld.to_owned(),
                                    &InterfaceType::TypeRef(&ty),
                                )?;
                                RuntimeVal::new(res_inst.new_val(Err(Some(inner_v.value())))?)
                            }
                            ([Ipld::Integer(1), Ipld::Null], None, _) => {
                                RuntimeVal::new(res_inst.new_val(Ok(None))?)
                            }
                            ([Ipld::Null, Ipld::Integer(1)], _, None) => {
                                RuntimeVal::new(res_inst.new_val(Err(None))?)
                            }
                            _ => Err(InterpreterError::IpldToWit(
                                "IPLD (as WIT result) has specific structure does does not match"
                                    .to_string(),
                            ))?,
                        }
                    } else {
                        Err(InterpreterError::IpldToWit("No match possible".to_string()))?
                    }
                }
                Ipld::Bool(v) => match interface_ty {
                    InterfaceType::Type(Type::Bool)
                    | InterfaceType::TypeRef(Type::Bool)
                    | InterfaceType::Any => RuntimeVal::new(Val::Bool(v)),
                    _ => Err(InterpreterError::IpldToWit(
                        "Expected conversion to bool".to_string(),
                    ))?,
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
                    // We need to handle odd issues with clients where 5.0
                    // becomes 5.
                    InterfaceType::Type(Type::Float32) => RuntimeVal::new(Val::Float32(v as f32)),
                    InterfaceType::Type(Type::Float64) => RuntimeVal::new(Val::Float64(v as f64)),
                    InterfaceType::Any
                    | InterfaceType::Type(Type::S64)
                    | InterfaceType::TypeRef(Type::S64) => RuntimeVal::new(Val::S64(v.try_into()?)),
                    _ => Err(InterpreterError::IpldToWit(
                        "Expected conversion to integer".to_string(),
                    ))?,
                },
                Ipld::Float(v) => match interface_ty {
                    InterfaceType::Type(Type::Float32) | InterfaceType::TypeRef(Type::Float32) => {
                        RuntimeVal::new(Val::Float32(v as f32))
                    }
                    _ => RuntimeVal::new(Val::Float64(v)),
                },
                Ipld::String(v) => match interface_ty.inner() {
                    Some(Type::Enum(enum_inst)) => enum_inst
                        .names()
                        .any(|name| name == v)
                        .then_some(RuntimeVal::new(enum_inst.new_val(&v)?))
                        .ok_or(InterpreterError::IpldToWit(
                            "IPLD string not an enum discriminant".to_string(),
                        ))?,
                    _ => RuntimeVal::new(Val::String(Box::from(v))),
                },
                Ipld::Bytes(v) => match interface_ty.inner() {
                    Some(Type::List(list_inst)) => {
                        let vec = v.into_iter().fold(vec![], |mut acc, elem| {
                            let RuntimeVal(value, _) = RuntimeVal::new(Val::U8(elem));
                            acc.push(value);
                            acc
                        });

                        RuntimeVal::new(list_inst.new_val(vec.into_boxed_slice())?)
                    }
                    _ => RuntimeVal::new(Val::String(Box::from(Base::Base64.encode(v)))),
                },
                Ipld::Link(v) => match v.version() {
                    cid::Version::V0 => RuntimeVal::new(Val::String(Box::from(
                        v.to_string_of_base(Base::Base58Btc)?,
                    ))),
                    cid::Version::V1 => RuntimeVal::new(Val::String(Box::from(
                        v.to_string_of_base(Base::Base32Lower)?,
                    ))),
                },
                Ipld::List(v) => match interface_ty.inner() {
                    Some(Type::List(list_inst)) => {
                        let vec = v.into_iter().try_fold(vec![], |mut acc, elem| {
                            let RuntimeVal(value, _) =
                                RuntimeVal::try_from(elem, &InterfaceType::Type(list_inst.ty()))?;
                            acc.push(value);
                            Ok::<_, InterpreterError>(acc)
                        })?;

                        RuntimeVal::new(list_inst.new_val(vec.into_boxed_slice())?)
                    }
                    Some(Type::Tuple(tuple_inst)) => {
                        let fields = tuple_inst.types().zip(v.into_iter()).try_fold(
                            vec![],
                            |mut acc, (ty, elem)| {
                                let RuntimeVal(value, _) =
                                    RuntimeVal::try_from(elem, &InterfaceType::TypeRef(&ty))?;
                                acc.push(value);
                                Ok::<_, InterpreterError>(acc)
                            },
                        )?;

                        RuntimeVal::new(tuple_inst.new_val(fields.into())?)
                    }
                    Some(Type::Flags(flags_inst)) => {
                        let flags = v.iter().try_fold(vec![], |mut acc, elem| {
                            if let Ipld::String(flag) = elem {
                                acc.push(flag.as_ref());
                                Ok::<_, InterpreterError>(acc)
                            } else {
                                Err(InterpreterError::IpldToWit(
                                    "IPLD (as flags) must contain only strings".to_string(),
                                ))
                            }
                        })?;

                        RuntimeVal::new(flags_inst.new_val(flags.as_slice())?)
                    }
                    _ => v
                        .into_iter()
                        .fold_while(Ok(RuntimeVal::new(Val::Bool(false))), |_acc, elem| {
                            match RuntimeVal::try_from(elem, interface_ty) {
                                Ok(runtime_val) => Done(Ok(runtime_val)),
                                Err(e) => Done(Err(e)),
                            }
                        })
                        .into_inner()?,
                },
                Ipld::Map(v) => match interface_ty.inner() {
                    Some(Type::List(list_inst)) => {
                        let tuple_inst = matches!(list_inst.ty(), Type::Tuple(_))
                            .then_some(list_inst.ty().unwrap_tuple())
                            .ok_or_else(|| InterpreterError::TypeMismatch {
                                expected: "<tuple>".to_string(),
                                given: format!("{:#?}", list_inst.ty()).into(),
                            })?
                            .to_owned();

                        let ty = tuple_inst.types().next().ok_or_else(|| {
                            InterpreterError::IpldToWit(
                                "IPLD map (for list) must have tuples of two elements".to_string(),
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
                    Some(Type::Variant(variant_inst)) => {
                        let (discriminant, v_inner) = v.first_key_value().ok_or_else(|| {
                            InterpreterError::IpldToWit(
                                "IPLD map (as variant) must have at least one key".to_string(),
                            )
                        })?;

                        match variant_inst.cases().find(|case| case.name == *discriminant) {
                            Some(case) => {
                                let opt_ty = case.ty;
                                if let Some(ty) = opt_ty {
                                    let RuntimeVal(value, _) = RuntimeVal::try_from(
                                        v_inner.to_owned(),
                                        &InterfaceType::TypeRef(&ty),
                                    )?;
                                    RuntimeVal::new(variant_inst.new_val(case.name, Some(value))?)
                                } else {
                                    RuntimeVal::new(variant_inst.new_val(case.name, None)?)
                                }
                            }
                            None => Err(InterpreterError::IpldToWit(
                                "IPLD map key does not match any variant case".to_string(),
                            ))?,
                        }
                    }
                    Some(Type::Record(record_inst)) => {
                        let fields =
                            record_inst
                                .fields()
                                .try_fold(IndexMap::new(), |mut acc, field| {
                                    if let Some((k, v_inner)) = v.get_key_value(field.name) {
                                        if field.name == *k {
                                            let RuntimeVal(value, _) = RuntimeVal::try_from(
                                                v_inner.to_owned(),
                                                &InterfaceType::TypeRef(&field.ty),
                                            )?;
                                            acc.insert(field.name, value);
                                            Ok::<_, InterpreterError>(acc)
                                        } else {
                                            Err(InterpreterError::IpldToWit(
                                                "IPLD map (as record) key does not match any record field"
                                                    .to_string(),
                                            ))
                                        }
                                    } else {
                                        Err(InterpreterError::IpldToWit(
                                            "IPLD map key does not match any record field"
                                                .to_string(),
                                        ))
                                    }
                                })?;

                        RuntimeVal::new(record_inst.new_val(fields)?)
                    }
                    ty => Err(InterpreterError::TypeMismatch {
                        expected: "<list|variant|record>".to_string(),
                        given: ty.map(|t| format!("{t:#?}")),
                    })?,
                },
            };

            Ok(dyn_type)
        })
    }
}

impl TryFrom<RuntimeVal> for Ipld {
    type Error = InterpreterError;

    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        fn cid(s: &str) -> Result<Cid, cid::Error> {
            Cid::try_from(s)
        }
        stacker::maybe_grow(DEFAULT_RED_ZONE, DEFAULT_EXTRA_STACK, || {
            let ipld = match val {
                RuntimeVal(Val::Char(c), _) => Ipld::String(c.to_string()),
                RuntimeVal(Val::String(v), _) => match v.to_string() {
                    s if s.eq("null") => Ipld::Null,
                    s => {
                        if let Ok(cid) = cid(&s) {
                            Ipld::Link(cid)
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
                RuntimeVal(Val::Variant(v), _) => {
                    let inner = if let Some(payload) = v.payload() {
                        Ipld::try_from(RuntimeVal::new(payload.to_owned()))?
                    } else {
                        Ipld::Null
                    };

                    Ipld::from(BTreeMap::from([(v.discriminant().to_string(), inner)]))
                }
                RuntimeVal(Val::Record(v), _) => {
                    let inner = v.fields().try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                        let ipld = Ipld::try_from(RuntimeVal::new(v.to_owned()))?;
                        acc.insert(k.to_string(), ipld);
                        Ok::<_, Self::Error>(acc)
                    })?;
                    Ipld::Map(inner)
                }
                RuntimeVal(Val::Option(opt), _) => {
                    if let Some(v) = opt.value() {
                        Ipld::try_from(RuntimeVal::new(v.to_owned()))?
                    } else {
                        Ipld::Null
                    }
                }
                RuntimeVal(Val::Result(res), _) => match res.value() {
                    Ok(Some(v)) => Ipld::List(vec![
                        Ipld::try_from(RuntimeVal::new(v.to_owned()))?,
                        Ipld::Null,
                    ]),
                    Ok(None) => Ipld::List(vec![Ipld::Integer(1), Ipld::Null]),
                    Err(Some(v)) => Ipld::List(vec![
                        Ipld::Null,
                        Ipld::try_from(RuntimeVal::new(v.to_owned()))?,
                    ]),
                    Err(None) => Ipld::List(vec![Ipld::Null, Ipld::Integer(1)]),
                },
                RuntimeVal(Val::Tuple(v), _) => {
                    let inner = v.values().iter().try_fold(vec![], |mut acc, elem| {
                        let ipld = Ipld::try_from(RuntimeVal::new(elem.to_owned()))?;
                        acc.push(ipld);
                        Ok::<_, Self::Error>(acc)
                    })?;
                    Ipld::List(inner)
                }
                RuntimeVal(Val::Flags(v), _) => {
                    let inner = v.flags().try_fold(vec![], |mut acc, flag| {
                        acc.push(Ipld::String(flag.to_string()));
                        Ok::<_, Self::Error>(acc)
                    })?;
                    Ipld::List(inner)
                }
                RuntimeVal(Val::Enum(v), _) => Ipld::String(v.discriminant().to_string()),
                // Rest of Wit types are unhandled going to Ipld.
                v => Err(InterpreterError::WitToIpld(format!("{v:#?}").into()))?,
            };

            Ok(ipld)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;
    use libipld::multihash::{Code, MultihashDigest};

    const RAW: u64 = 0x55;

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
    fn try_integer_to_float() {
        let ipld_in = Ipld::Integer(5);
        let ipld_out = Ipld::Float(5.0);
        let runtime_float = RuntimeVal::new(Val::Float32(5.0));

        let ty = test_utils::component::setup_component_with_param(
            "float32".to_string(),
            &[test_utils::component::Param(
                test_utils::component::Type::F32,
                None,
            )],
        );

        assert_eq!(
            RuntimeVal::try_from(ipld_in.clone(), &InterfaceType::Type(ty)).unwrap(),
            runtime_float
        );

        assert_eq!(Ipld::try_from(runtime_float).unwrap(), ipld_out);
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

        let ty = test_utils::component::setup_component("(list u8)".to_string(), 8);
        let val_list = ty
            .unwrap_list()
            .new_val(Box::new([
                Val::U8(104),
                Val::U8(101),
                Val::U8(108),
                Val::U8(108),
                Val::U8(48),
            ]))
            .unwrap();
        let runtime = RuntimeVal::new(val_list);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &InterfaceType::Type(ty)).unwrap(),
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

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_variant_roundtrip() {
        let ipld1 = Ipld::from(BTreeMap::from([("B".into(), Ipld::Integer(22))]));
        let ipld2 = Ipld::from(BTreeMap::from([("foo-bar-baz".into(), Ipld::Integer(-22))]));
        let ipld3 = Ipld::from(BTreeMap::from([("C".into(), Ipld::Null)]));

        let ty = test_utils::component::setup_component(
            r#"(variant (case "foo-bar-baz" s32) (case "B" u32) (case "C"))"#.to_string(),
            8,
        );
        let interface_ty = InterfaceType::Type(ty.clone());
        let ty_var = ty.unwrap_variant();

        let val1 = ty_var.clone().new_val("B", Some(Val::U32(22))).unwrap();
        let runtime1 = RuntimeVal::new(val1);
        assert_eq!(
            RuntimeVal::try_from(ipld1.clone(), &interface_ty).unwrap(),
            runtime1
        );
        assert_eq!(Ipld::try_from(runtime1).unwrap(), ipld1);

        let val2 = ty_var.new_val("foo-bar-baz", Some(Val::S32(-22))).unwrap();
        let runtime2 = RuntimeVal::new(val2);
        assert_eq!(
            RuntimeVal::try_from(ipld2.clone(), &interface_ty).unwrap(),
            runtime2
        );
        assert_eq!(Ipld::try_from(runtime2).unwrap(), ipld2);

        let val3 = ty_var.new_val("C", None).unwrap();
        let runtime3 = RuntimeVal::new(val3);
        assert_eq!(
            RuntimeVal::try_from(ipld3.clone(), &interface_ty).unwrap(),
            runtime3
        );
        assert_eq!(Ipld::try_from(runtime3).unwrap(), ipld3);
    }

    #[test]
    fn try_record_roundtrip() {
        let ipld = Ipld::Map(BTreeMap::from([
            ("foo-bar-baz".into(), Ipld::Integer(-22)),
            ("b".into(), Ipld::String("Hello!".into())),
        ]));

        let ty = test_utils::component::setup_component(
            r#"(record (field "foo-bar-baz" s32) (field "b" string))"#.to_string(),
            12,
        );

        let interface_ty = InterfaceType::Type(ty.clone());
        let ty_rec = ty.unwrap_record();

        let fields = ty_rec.fields().fold(IndexMap::new(), |mut acc, field| {
            let val = match field.name {
                "foo-bar-baz" => Val::S32(-22),
                "b" => Val::String(Box::from("Hello!")),
                _ => unreachable!(),
            };
            acc.insert(field.name, val);
            acc
        });

        let val = ty_rec.new_val(fields).unwrap();

        let runtime = RuntimeVal::new(val);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &interface_ty).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_option_roundtrip() {
        let ipld1 = Ipld::Null;
        let ipld2 = Ipld::String("Hello!".into());

        let ty = test_utils::component::setup_component("(option string)".to_string(), 12);
        let interface_ty = InterfaceType::Type(ty.clone());

        let val1 = ty.unwrap_option().new_val(None).unwrap();
        let runtime1 = RuntimeVal::new(val1);

        let val2 = ty
            .unwrap_option()
            .new_val(Some(Val::String(Box::from("Hello!"))))
            .unwrap();
        let runtime2 = RuntimeVal::new(val2);

        assert_eq!(
            RuntimeVal::try_from(ipld1.clone(), &interface_ty).unwrap(),
            runtime1
        );
        assert_eq!(Ipld::try_from(runtime1).unwrap(), ipld1);

        assert_eq!(
            RuntimeVal::try_from(ipld2.clone(), &interface_ty).unwrap(),
            runtime2
        );
        assert_eq!(Ipld::try_from(runtime2).unwrap(), ipld2);
    }

    #[test]
    fn try_result_roundtrip() {
        let ok_ipld = Ipld::List(vec![Ipld::String("Hello!".into()), Ipld::Null]);
        let err_ipld = Ipld::List(vec![Ipld::Null, Ipld::String("Hello!".into())]);
        let ok_res_ipld = Ipld::List(vec![Ipld::Integer(1), Ipld::Null]);
        let err_res_ipld = Ipld::List(vec![Ipld::Null, Ipld::Integer(1)]);

        let ty1 = test_utils::component::setup_component("(result string)".to_string(), 12);
        let interface_ty1 = InterfaceType::Type(ty1.clone());

        let ty2 = test_utils::component::setup_component(
            "(result string (error string))".to_string(),
            12,
        );
        let interface_ty2 = InterfaceType::Type(ty2.clone());

        let ty3 = test_utils::component::setup_component("(result)".to_string(), 4);
        let interface_ty3 = InterfaceType::Type(ty3.clone());

        let val1 = ty1
            .unwrap_result()
            .new_val(Ok(Some(Val::String(Box::from("Hello!")))))
            .unwrap();
        let runtime1 = RuntimeVal::new(val1);

        let val2 = ty2
            .unwrap_result()
            .new_val(Err(Some(Val::String(Box::from("Hello!")))))
            .unwrap();
        let runtime2 = RuntimeVal::new(val2);

        let val3 = ty3.unwrap_result().new_val(Ok(None)).unwrap();
        let runtime3 = RuntimeVal::new(val3);

        let val4 = ty3.unwrap_result().new_val(Err(None)).unwrap();
        let runtime4 = RuntimeVal::new(val4);

        let val5 = ty1.unwrap_result().new_val(Err(None)).unwrap();
        let runtime5 = RuntimeVal::new(val5);

        assert_eq!(
            RuntimeVal::try_from(ok_ipld.clone(), &interface_ty1).unwrap(),
            runtime1
        );
        assert_eq!(Ipld::try_from(runtime1).unwrap(), ok_ipld);

        assert_eq!(
            RuntimeVal::try_from(err_ipld.clone(), &interface_ty2).unwrap(),
            runtime2
        );
        assert_eq!(Ipld::try_from(runtime2).unwrap(), err_ipld);

        assert_eq!(
            RuntimeVal::try_from(ok_res_ipld.clone(), &interface_ty3).unwrap(),
            runtime3
        );
        assert_eq!(Ipld::try_from(runtime3).unwrap(), ok_res_ipld);

        assert_eq!(
            RuntimeVal::try_from(err_res_ipld.clone(), &interface_ty3).unwrap(),
            runtime4
        );
        assert_eq!(Ipld::try_from(runtime4).unwrap(), err_res_ipld);

        assert_eq!(
            RuntimeVal::try_from(err_res_ipld.clone(), &interface_ty1).unwrap(),
            runtime5
        );
        assert_eq!(Ipld::try_from(runtime5).unwrap(), err_res_ipld);
    }

    #[test]
    fn try_tuple_roundtrip() {
        let ipld = Ipld::List(vec![
            Ipld::Integer(22),
            Ipld::String("Hello!".into()),
            Ipld::Bool(true),
        ]);

        let ty = test_utils::component::setup_component("(tuple s32 string bool)".to_string(), 16);
        let interface_ty = InterfaceType::Type(ty.clone());

        let val = ty
            .unwrap_tuple()
            .new_val(Box::new([
                Val::S32(22),
                Val::String(Box::from("Hello!")),
                Val::Bool(true),
            ]))
            .unwrap();

        let runtime = RuntimeVal::new(val);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &interface_ty).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_enum_roundtrip() {
        let ipld1 = Ipld::String("foo-bar-baz".into());
        let ipld2 = Ipld::String("b".into());

        let ty =
            test_utils::component::setup_component(r#"(enum "foo-bar-baz" "b")"#.to_string(), 4);
        let interface_ty = InterfaceType::Type(ty.clone());

        let val1 = ty.unwrap_enum().new_val("foo-bar-baz").unwrap();
        let runtime1 = RuntimeVal::new(val1);

        let val2 = ty.unwrap_enum().new_val("b").unwrap();
        let runtime2 = RuntimeVal::new(val2);

        assert_eq!(
            RuntimeVal::try_from(ipld1.clone(), &interface_ty).unwrap(),
            runtime1
        );
        assert_eq!(Ipld::try_from(runtime1).unwrap(), ipld1);

        assert_eq!(
            RuntimeVal::try_from(ipld2.clone(), &interface_ty).unwrap(),
            runtime2
        );
        assert_eq!(Ipld::try_from(runtime2).unwrap(), ipld2);
    }

    #[test]
    fn try_flags_roundtrip() {
        let ipld = Ipld::List(vec![
            Ipld::String("foo-bar-baz".into()),
            Ipld::String("B".into()),
            Ipld::String("C".into()),
        ]);

        let ty = test_utils::component::setup_component(
            r#"(flags "foo-bar-baz" "B" "C")"#.to_string(),
            4,
        );
        let interface_ty = InterfaceType::Type(ty.clone());

        let val = ty
            .unwrap_flags()
            .new_val(&["foo-bar-baz", "B", "C"])
            .unwrap();

        let runtime = RuntimeVal::new(val);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), &interface_ty).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }
}
