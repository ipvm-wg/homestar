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
