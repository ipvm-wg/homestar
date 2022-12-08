use cid::Cid;
use json::JsonValue;
use signature::Signature;
use std::{collections::HashMap, marker::PhantomData};
use ucan::ucan::Ucan;
use url::Url;

pub struct Closure {
    pub resource: Url,
    pub action: String,
    pub inputs: Input,
}

pub struct Task {
    pub closure: Closure,
    pub resources: Resources,
    pub metadata: JsonValue,
    pub secret: Option<bool>,
}

pub struct Resources {
    pub fuel: u32,
    pub time: u32,
}

pub struct Batch(HashMap<TaskLabel, Task>);

pub struct Invocation<Sig: Signature> {
    pub run: Batch,
    pub sig: Sig,
    pub meta: JsonValue, // Just me being lazy, but also "not wrong"
    pub prf: [Link<Ucan>],
}

pub struct Promise {
    pub invoked_task: InvokedTaskPointer,
    pub branch_selector: Option<Status>,
}

pub enum Status {
    Success,
    Failure,
}

pub enum InvocationPointer {
    Remote(Cid),
    Local,
}

pub struct InvokedTaskPointer {
    pub invocation: InvocationPointer,
    pub label: TaskLabel,
}

pub enum Input {
    Wasm(wasmer::Value),
    Deferred(Promise),
    Reference(Cid),
    List(Vec<Input>),
    Map(HashMap<String, Input>),
}

pub struct Link<T>(Cid, PhantomData<T>);
pub struct TaskLabel(String);

/////////////////////////////////////////

// Now for a DAG and some light type checking ;)

//    const Sha3_256: u64 = 0x16;
//    let digest_bytes = [
//        0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
//        0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
//        0xf2, 0x4e, 0x39, 0x38
//    ];

//    let multihash = Multihash::from_bytes(&digest_bytes).unwrap();

//    Job {
//        tasks: BTreeMap::from([
//            (TaskLabel("left"), PureTask(Pure{
//                wasm: Cid.new_v0(...),
//                inputs: [
//                    WasmParam(Value::I32(1)),
//                    WasmParam(Value::I32(2))
//                ]
//            })),
//            (TaskLabel("right"), PureTask(Pure{
//                wasm: Cid.new_v0(...),
//                inputs: [
//                    Absolute(Cid.new_v0(multihash))
//                ]
//            })),
//            (TaskLabel("end"), PureTask(Pure{
//                wasm: Cid.new_v0(...),
//                inputs: [
//                    Relative(TaskLabel("left")),
//                    WasmParam(Value::I32(42)),
//                    Relative(TaskLabel("right"))
//                ]
//            }))
//        ])
//    }
