//! Convert (bidirectionally) between [Ipld] and [wasmtime::component::Val]s
//! and [wasmtime::component::Type]s.

use anyhow::{anyhow, Result};
use itertools::Itertools;
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
use std::{collections::BTreeMap, str};
use wasmtime::component::{Type, Val};

/// Interface-type wrapper over wasmtime component [wasmtime::component::Type].
#[derive(Clone, Debug)]
pub enum InterfaceType {
    /// Works for `any` [wasmtime::component::Type].
    Any,
    /// Wraps [wasmtime::component::Type].
    Type(Type),
}

impl InterfaceType {
    fn into_inner(self) -> Option<Type> {
        match self {
            InterfaceType::Type(ty) => Some(ty),
            _ => None,
        }
    }
}

impl<'a> From<&'a Type> for InterfaceType {
    fn from(typ: &'a Type) -> Self {
        match typ {
            Type::List(_)
            | Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Float32
            | Type::Float64 => InterfaceType::Type(typ.to_owned()),
            _ => InterfaceType::Any,
        }
    }
}

/// Wrapper-type for runtime [wasmtime::component::Val].
#[derive(Debug, PartialEq)]
pub struct RuntimeVal(pub Val);

impl RuntimeVal {
    /// Return [wasmtime::component::Val] from [RuntimeVal] wrapper.
    pub fn into_inner(self) -> Val {
        self.0
    }

    /// Convert from [Ipld] to [RuntimeVal] with a given [InterfaceType].
    pub fn try_from(ipld: Ipld, interface_ty: InterfaceType) -> Result<Self> {
        // TODO: Configure for recursion.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            let dyn_type = match ipld {
                Ipld::Null => RuntimeVal(Val::String(Box::from("null"))),
                Ipld::Bool(v) => RuntimeVal(Val::Bool(v)),
                Ipld::Integer(v) => match interface_ty {
                    InterfaceType::Type(Type::U8) => RuntimeVal(Val::U8(v.try_into()?)),
                    InterfaceType::Type(Type::U16) => RuntimeVal(Val::U16(v.try_into()?)),
                    InterfaceType::Type(Type::U32) => RuntimeVal(Val::U32(v.try_into()?)),
                    InterfaceType::Type(Type::U64) => RuntimeVal(Val::U64(v.try_into()?)),
                    InterfaceType::Type(Type::S8) => RuntimeVal(Val::S8(v.try_into()?)),
                    InterfaceType::Type(Type::S16) => RuntimeVal(Val::S16(v.try_into()?)),
                    InterfaceType::Type(Type::S32) => RuntimeVal(Val::S32(v.try_into()?)),
                    _ => RuntimeVal(Val::S64(v.try_into()?)),
                },
                Ipld::Float(v) => match interface_ty {
                    InterfaceType::Type(Type::Float32) => RuntimeVal(Val::Float32(v as f32)),
                    _ => RuntimeVal(Val::Float64(v)),
                },
                Ipld::String(v) => RuntimeVal(Val::String(Box::from(v))),
                Ipld::Bytes(v) => RuntimeVal(Val::String(Box::from(Base::Base64.encode(v)))),
                Ipld::Link(v) => match v.version() {
                    cid::Version::V0 => RuntimeVal(Val::String(Box::from(
                        v.to_string_of_base(Base::Base58Btc)?,
                    ))),
                    cid::Version::V1 => RuntimeVal(Val::String(Box::from(
                        v.to_string_of_base(Base::Base32Lower)?,
                    ))),
                },
                Ipld::List(v) => {
                    let vec = v
                        .into_iter()
                        .map(|elem| RuntimeVal::try_from(elem, interface_ty.clone()))
                        .fold_ok(vec![], |mut acc, elem| {
                            acc.push(elem.into_inner());
                            acc
                        })?;

                    let inner = interface_ty
                        .into_inner()
                        .ok_or_else(|| anyhow!("component type mismatch: expected <list>"))?;

                    let list_inst = matches!(inner, Type::List(_))
                        .then_some(inner.unwrap_list())
                        .ok_or_else(|| anyhow!("{inner:?} must be a <list>"))?;

                    RuntimeVal(list_inst.new_val(vec.into_boxed_slice())?)
                }
                Ipld::Map(v) => {
                    let inner = interface_ty
                        .into_inner()
                        .ok_or_else(|| anyhow!("component type mismatch: expected <List>"))?;

                    let list_inst = matches!(inner, Type::List(_))
                        .then_some(inner.unwrap_list())
                        .ok_or_else(|| anyhow!("{inner:?} must be a <list>"))?;

                    let tuple_inst = matches!(list_inst.ty(), Type::Tuple(_))
                        .then_some(list_inst.ty().unwrap_tuple())
                        .ok_or_else(|| anyhow!("{inner:?} must be a <list>"))?
                        .to_owned();

                    let vec = v
                        .into_iter()
                        .map(|(key, elem)| {
                            match RuntimeVal::try_from(elem, InterfaceType::Type(inner.clone())) {
                                Ok(value) => {
                                    let tuple = Box::new([Val::String(Box::from(key)), value.0]);
                                    tuple_inst.new_val(tuple)
                                }
                                Err(e) => Err(anyhow!(e)),
                            }
                        })
                        .fold_ok(vec![], |mut acc, tuple| {
                            acc.push(tuple);
                            acc
                        })?;

                    RuntimeVal(list_inst.new_val(vec.into_boxed_slice())?)
                }
            };

            Ok(dyn_type)
        })
    }
}

impl TryFrom<RuntimeVal> for Ipld {
    type Error = anyhow::Error;

    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        fn base_64_bytes(s: &str) -> Result<Vec<u8>, multibase::Error> {
            Base::Base64.decode(s)
        }
        fn cid(s: &str) -> Result<Cid, cid::Error> {
            Cid::try_from(s)
        }
        let ipld = match val {
            RuntimeVal(Val::String(v)) => match v.to_string() {
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
            RuntimeVal(Val::Bool(v)) => Ipld::Bool(v),
            RuntimeVal(Val::S8(v)) => Ipld::Integer(v.into()),
            RuntimeVal(Val::S16(v)) => Ipld::Integer(v.into()),
            RuntimeVal(Val::S32(v)) => Ipld::Integer(v.into()),
            RuntimeVal(Val::S64(v)) => Ipld::Integer(v.into()),
            RuntimeVal(Val::Float32(v)) => {
                // Convert to decimal for handling precision issues going from
                // f32 => f64.
                let dec =
                    Decimal::from_f32(v).ok_or_else(|| anyhow!("failed conversion to decimal"))?;
                Ipld::Float(
                    dec.to_f64()
                        .ok_or_else(|| anyhow!("failed conversion from decimal"))?,
                )
            }
            RuntimeVal(Val::Float64(v)) => Ipld::Float(v),
            RuntimeVal(Val::List(v)) if matches!(v.ty().ty(), Type::Tuple(_)) => {
                let inner = v
                    .iter()
                    .map(|elem| {
                        if let Val::Tuple(tup) = elem {
                            let tup_values = tup.values();
                            if let [Val::String(s), v] = tup_values {
                                match Ipld::try_from(RuntimeVal(v.to_owned())) {
                                    Ok(value) => Ok((s.to_string(), value)),
                                    Err(e) => Err(e),
                                }
                            } else {
                                Err(anyhow!("mismatched types: {:?}", tup_values))
                            }
                        } else {
                            Err(anyhow!("mismatched types: {elem:?}"))
                        }
                    })
                    .fold_ok(BTreeMap::new(), |mut acc, (k, v)| {
                        acc.insert(k, v);
                        acc
                    })?;

                Ipld::Map(inner)
            }
            RuntimeVal(Val::List(v)) => {
                let inner = v
                    .iter()
                    .map(|elem| Ipld::try_from(RuntimeVal(elem.to_owned())))
                    .fold_ok(vec![], |mut acc, elem| {
                        acc.push(elem);
                        acc
                    })?;

                Ipld::List(inner)
            }
            _ => Ipld::Null,
        };

        Ok(ipld)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;
    use libipld::multihash::{Code, MultihashDigest};
    use std::collections::BTreeMap;

    const RAW: u64 = 0x55;

    #[test]
    fn try_null_roundtrip() {
        let runtime_null = RuntimeVal(Val::String(Box::from("null")));

        assert_eq!(
            RuntimeVal::try_from(Ipld::Null, InterfaceType::Any).unwrap(),
            runtime_null
        );

        assert_eq!(Ipld::try_from(runtime_null).unwrap(), Ipld::Null);
    }

    #[test]
    fn try_bool_roundtrip() {
        let runtime_bool = RuntimeVal(Val::Bool(false));

        assert_eq!(
            RuntimeVal::try_from(Ipld::Bool(false), InterfaceType::Any).unwrap(),
            runtime_bool
        );

        assert_eq!(Ipld::try_from(runtime_bool).unwrap(), Ipld::Bool(false));
    }

    #[test]
    fn try_integer_any_roundtrip() {
        let ipld = Ipld::Integer(2828829);
        let runtime_int = RuntimeVal(Val::S64(2828829));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_integer_type_roundtrip() {
        let ipld = Ipld::Integer(8829);
        let runtime_int = RuntimeVal(Val::S16(8829));

        let ty = test_utils::component::setup_component("s16".to_string(), 4);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Type(ty)).unwrap(),
            runtime_int
        );

        assert_eq!(Ipld::try_from(runtime_int).unwrap(), ipld);
    }

    #[test]
    fn try_float_any_roundtrip() {
        let ipld = Ipld::Float(3883.20);
        let runtime_float = RuntimeVal(Val::Float64(3883.20));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
            runtime_float
        );

        assert_eq!(Ipld::try_from(runtime_float).unwrap(), ipld);
    }

    #[test]
    fn try_float_type_roundtrip() {
        let ipld = Ipld::Float(3883.20);
        let runtime_float = RuntimeVal(Val::Float32(3883.20));

        let ty = test_utils::component::setup_component_with_param(
            "float32".to_string(),
            &[test_utils::component::Param(
                test_utils::component::Type::F32,
                Some(0),
            )],
        );

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Type(ty)).unwrap(),
            runtime_float
        );

        assert_eq!(Ipld::try_from(runtime_float).unwrap(), ipld);
    }

    #[test]
    fn try_string_roundtrip() {
        let ipld = Ipld::String("Hello!".into());
        let runtime = RuntimeVal(Val::String(Box::from("Hello!")));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_bytes_roundtrip() {
        let bytes = b"hell0".to_vec();
        let ipld = Ipld::Bytes(bytes.clone());
        let encoded_cid = Base::Base64.encode(bytes);
        let runtime = RuntimeVal(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }

    #[test]
    fn try_cid_v1_roundtrip() {
        let h = Code::Sha2_256.digest(b"beep boop");
        let cid = Cid::new_v1(RAW, h);
        let ipld = Ipld::Link(cid);
        let encoded_cid = cid.to_string_of_base(Base::Base32Lower).unwrap();
        let runtime = RuntimeVal(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
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
        let runtime = RuntimeVal(Val::String(Box::from(encoded_cid)));

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Any).unwrap(),
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

        let runtime = RuntimeVal(val_list);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Type(ty)).unwrap(),
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

        let val_tuple1 = ty
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple1))
            .unwrap();

        let val_tuple2 = ty
            .unwrap_list()
            .ty()
            .unwrap_tuple()
            .new_val(Box::new(tuple2))
            .unwrap();

        let val_map = ty
            .unwrap_list()
            .new_val(Box::new([val_tuple1, val_tuple2]))
            .unwrap();

        let runtime = RuntimeVal(val_map);

        assert_eq!(
            RuntimeVal::try_from(ipld.clone(), InterfaceType::Type(ty)).unwrap(),
            runtime
        );

        assert_eq!(Ipld::try_from(runtime).unwrap(), ipld);
    }
}
