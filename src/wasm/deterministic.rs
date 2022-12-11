// use libipld::Ipld;
// use wasmer::Value;

// I32(i32),
// I64(i64),
// F32(f32),
// F64(f64),
// ExternRef(Option<ExternRef>),
// FuncRef(Option<Function>),
// V128(u128),

// fn try_from_ipld(ipld: Ipld) -> Result<Value, ()> {
//     match ipld {
//         Ipld::Bool(false) => Ok(wasmer::Value::I32(0)),
//         Ipld::Bool(true) => Ok(wasmer::Value::I32(1)),
//         Ipld::Integer(int128) => match i64::try_from(int128) {
//             Ok(i) => Ok(wasmer::Value::I64(i)),
//             _ => Err(()),
//         },
//         Ipld::Float(float64) => Err(()), // Nondeterministic; else Ok(wasmer::Value::F64(float64)),
//
//         // Now have to pick a representation. C conventions
//         // https://docs.rs/wasmer/3.0.2/wasmer/enum.CallingConvention.html#variant.WasmBasicCAbi
//         Ipld::String(s) => todo!(),
//         Ipld::Bytes(vecU8) => todo!(),
//         Ipld::List(vecIpld) => todo!(),
//         Ipld::Map(assoc) => todo!(),
//         Ipld::Link(cid) => todo!(),
//         Ipld::Null => Err(()),
//     }
// }
//
// pub enum IpvmIR {
//     Lit(IpvmLiteral),
//     Ptr(IpvmPointer),
//     Ctl(IpvmControl),
// }
//
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub enum IpvmLiteral {
//     Boolean(bool),
//     Integer(i64),
// }
//
// impl Into<wasmer::Value> for IpvmLiteral {
//     fn into(wasm: wasmer::Value) -> Self {
//         match wasm {
//
//         }
//     }
// }

// pub enum IpvmPointer {
//     Defer(Promise),
// }
//
// pub enum IpvmControl {}
