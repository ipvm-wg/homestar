use cid::Cid;
use core::ops::ControlFlow;
use libipld::{cid::multibase::Base, Ipld};
use std::collections::BTreeMap;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub resource: Url,
    pub action: Action,
    pub inputs: Input,
}

impl TryFrom<Ipld> for Closure {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => assoc
                .get("with")
                .and_then(|res_ipld| match res_ipld {
                    Ipld::Link(cid) => match cid.to_string_of_base(Base::Base32HexLower) {
                        Ok(txt) => {
                            let ipfs_url: String = format!("{}{}", "ipfs://", txt);
                            Url::parse(ipfs_url.as_str()).ok()
                        }
                        _ => None,
                    },
                    Ipld::String(txt) => Url::parse(txt.as_str()).ok(),
                    _ => None,
                })
                .and_then(|resource| {
                    assoc.get("do").and_then(|ipld| {
                        Action::try_from(ipld.clone()).ok().and_then(|action| {
                            assoc.get("inputs").and_then(|ipld| {
                                Some(Closure {
                                    resource,
                                    action,
                                    inputs: Input::from(ipld.clone()),
                                })
                            })
                        })
                    })
                })
                .ok_or(()),

            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    IpldData { ipld: Ipld },
    Deferred { promise: Promise },
}

impl Into<Ipld> for Input {
    fn into(self) -> Ipld {
        match self {
            Input::IpldData { ipld } => ipld,
            Input::Deferred { promise } => Promise::into(promise),
        }
    }
}

impl From<Ipld> for Input {
    fn from(ipld: Ipld) -> Input {
        match ipld {
            Ipld::Map(ref map) => {
                if map.len() != 1 {
                    return Input::IpldData { ipld };
                }
                match map.get("ucan/ok") {
                    Some(Ipld::List(pointer)) => {
                        if let Ok(invoked_task) =
                            InvokedTaskPointer::try_from(Ipld::List(pointer.clone()))
                        {
                            Input::Deferred {
                                promise: Promise {
                                    branch_selector: Some(Status::Success),
                                    invoked_task,
                                },
                            }
                        } else {
                            Input::IpldData { ipld }
                        }
                    }

                    _ => Input::IpldData { ipld },
                }
            }
            _ => Input::IpldData { ipld },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action(String);

impl Into<Ipld> for Action {
    fn into(self) -> Ipld {
        match self {
            Action(string) => Ipld::String(string),
        }
    }
}

impl TryFrom<Ipld> for Action {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(txt) => Ok(Action(txt)),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub closure: Closure,
    pub resources: Resources,
    pub metadata: Ipld,
    pub secret: Option<bool>,
}

impl Into<Ipld> for Task {
    fn into(self) -> Ipld {
        let secret_flag = match self.secret {
            None => Ipld::Null,
            Some(b) => Ipld::Bool(b),
        };

        Ipld::Map(BTreeMap::from([
            (
                "with".to_string(),
                Ipld::String(self.closure.resource.into()),
            ),
            ("do".to_string(), self.closure.action.into()),
            ("inputs".to_string(), self.closure.inputs.into()),
            ("resources".to_string(), self.resources.into()),
            ("secret".to_string(), secret_flag),
        ]))
    }
}

impl TryFrom<Ipld> for Task {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(ref assoc) => {
                let res = match assoc.get("resources") {
                    Some(v) => v.clone(),
                    _ => Ipld::Map(BTreeMap::new()),
                };

                let fuel: Option<u32> =
                    res.get("fuel")
                        .map_err(|_| ())
                        .and_then(|ipld_fuel| match ipld_fuel {
                            Ipld::Integer(int) => Ok(u32::try_from(*int).ok()),
                            _ => Err(()),
                        })?;

                let time: Option<u32> =
                    res.get("time")
                        .map_err(|_| ())
                        .and_then(|ipld_fuel| match ipld_fuel {
                            Ipld::Integer(int) => Ok(u32::try_from(*int).ok()),
                            _ => Err(()),
                        })?;

                let metadata: Ipld = match assoc.get("meta") {
                    Some(ipld) => ipld.clone(),
                    None => Ipld::Null,
                };

                // Is it secret? Is it safe?!
                let secret: Option<bool> =
                    assoc.get("secret").ok_or(()).and_then(|ipld| match ipld {
                        Ipld::Bool(b) => Ok(Some(*b)),
                        Ipld::Null => Ok(None),
                        _ => Err(()),
                    })?;

                Ok(Task {
                    closure: Closure::try_from(ipld)?,
                    resources: Resources { time, fuel },
                    metadata,
                    secret,
                })
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resources {
    pub fuel: Option<u32>,
    pub time: Option<u32>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            fuel: None,
            time: None,
        }
    }
}

impl Into<Ipld> for Resources {
    fn into(self) -> Ipld {
        let fuel_ipld = match self.fuel {
            None => Ipld::Null,
            Some(int) => Ipld::from(int),
        };

        let time_ipld = match self.time {
            None => Ipld::Null,
            Some(int) => Ipld::from(int),
        };

        Ipld::Map(BTreeMap::from([
            ("fuel".to_string(), fuel_ipld),
            ("time".to_string(), time_ipld),
        ]))
    }
}

impl TryFrom<Ipld> for Resources {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(map) => {
                let fuel: Option<u32> = match map.get("fuel") {
                    Some(Ipld::Integer(int)) => u32::try_from(*int).ok(),
                    _ => None,
                };

                let time: Option<u32> = match map.get("time") {
                    Some(Ipld::Integer(int)) => u32::try_from(*int).ok(),
                    _ => None,
                };

                Ok(Resources { fuel, time })
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Batch(BTreeMap<TaskLabel, Task>);

impl Into<Ipld> for Batch {
    fn into(self) -> Ipld {
        match self {
            Batch(assoc) => {
                let mut batch = BTreeMap::new();

                assoc.iter().for_each(|(TaskLabel(label), task)| {
                    batch.insert(label.clone(), task.clone().into());
                });

                Ipld::Map(batch)
            }
        }
    }
}

impl TryFrom<Ipld> for Batch {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                let mut batch = BTreeMap::new();

                let flow =
                    assoc
                        .iter()
                        .try_for_each(|(key, value)| match Task::try_from(value.clone()) {
                            Ok(task) => {
                                batch.insert(TaskLabel(key.to_string()), task);
                                ControlFlow::Continue(())
                            }
                            _ => ControlFlow::Break(()),
                        });

                match flow {
                    ControlFlow::Continue(_) => Ok(Batch(batch)),
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Invocation {
    pub run: Batch,
    // pub sig: Sig,
    pub meta: Ipld,
    // pub prf: Vec<Link<Ucan>>,
}

impl Into<Ipld> for Invocation {
    fn into(self) -> Ipld {
        Ipld::Map(BTreeMap::from([
            ("run".to_string(), self.run.clone().into()),
            ("meta".to_string(), self.meta),
        ]))
    }
}

impl TryFrom<Ipld> for Invocation {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                let run: Batch = match assoc.get("run") {
                    Some(ipld) => Batch::try_from(ipld.clone()),
                    _ => Err(()),
                }?;

                let meta = match assoc.get("meta") {
                    Some(ipld) => ipld.clone(),
                    None => Ipld::Null,
                };

                Ok(Invocation { run, meta })
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promise {
    pub invoked_task: InvokedTaskPointer,
    pub branch_selector: Option<Status>,
}

impl Into<Ipld> for Promise {
    fn into(self) -> Ipld {
        let key: String = match self.branch_selector {
            Some(Status::Success) => "ucan/ok".to_string(),
            Some(Status::Failure) => "ucan/err".to_string(),
            None => "ucan/promise".to_string(),
        };

        Ipld::Map(BTreeMap::from([(key, self.invoked_task.into())]))
    }
}

impl TryFrom<Ipld> for Promise {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                if assoc.len() != 1 {
                    return Err(());
                }

                let (key, value) = assoc.iter().next().unwrap();
                let invoked_task = InvokedTaskPointer::try_from(value.clone())?;

                let branch_selector = match key.as_str() {
                    "ucan/ok" => Ok(Some(Status::Success)),
                    "ucan/err" => Ok(Some(Status::Failure)),
                    "ucan/promise" => Ok(None),
                    _ => Err(()),
                }?;

                return Ok(Promise {
                    invoked_task,
                    branch_selector,
                });
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationPointer {
    Remote(Cid),
    Local,
}

impl Into<Ipld> for InvocationPointer {
    fn into(self) -> Ipld {
        match self {
            InvocationPointer::Local => Ipld::String("/".to_string()),
            InvocationPointer::Remote(cid) => Ipld::Link(cid),
        }
    }
}

impl TryFrom<Ipld> for InvocationPointer {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => match s.as_str() {
                "/" => Ok(InvocationPointer::Local),
                other => match Cid::try_from(other) {
                    Ok(cid) => Ok(InvocationPointer::Remote(cid)),
                    Err(_) => Err(()),
                },
            },
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvokedTaskPointer {
    pub invocation: InvocationPointer,
    pub label: TaskLabel,
}

impl Into<Ipld> for InvokedTaskPointer {
    fn into(self) -> Ipld {
        Ipld::List(vec![self.invocation.into(), self.label.into()])
    }
}

impl TryFrom<Ipld> for InvokedTaskPointer {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::List(list) => match &list[..] {
                [Ipld::String(s), Ipld::String(label)] => match s.as_str() {
                    "/" => Ok(InvokedTaskPointer {
                        invocation: InvocationPointer::Local,
                        label: TaskLabel(label.to_string()),
                    }),
                    _ => Err(()),
                },

                [Ipld::Link(ptr), Ipld::String(label)] => Ok(InvokedTaskPointer {
                    invocation: InvocationPointer::Remote(*ptr),
                    label: TaskLabel(label.to_string()),
                }),

                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskLabel(String);

impl Into<Ipld> for TaskLabel {
    fn into(self) -> Ipld {
        match self {
            TaskLabel(txt) => Ipld::String(txt.to_string()),
        }
    }
}

impl TryFrom<Ipld> for TaskLabel {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(label) => Ok(TaskLabel(label)),
            _ => Err(()),
        }
    }
}

//////////////////////////////////////

//Now for a DAG and some light type checking ;)
//
//   const Sha3_256: u64 = 0x16;
//   let digest_bytes = [
//       0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
//       0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
//       0xf2, 0x4e, 0x39, 0x38
//   ];
//
//   let multihash = Multihash::from_bytes(&digest_bytes).unwrap();
//
//   Job {
//       tasks: BTreeMap::from([
//           (TaskLabel("left"), PureTask(Pure{
//               wasm: Cid.new_v0(...),
//               inputs: [
//                   WasmParam(Value::I32(1)),
//                   WasmParam(Value::I32(2))
//               ]
//           })),
//           (TaskLabel("right"), PureTask(Pure{
//               wasm: Cid.new_v0(...),
//               inputs: [
//                   Absolute(Cid.new_v0(multihash))
//               ]
//           })),
//           (TaskLabel("end"), PureTask(Pure{
//               wasm: Cid.new_v0(...),
//               inputs: [
//                   Relative(TaskLabel("left")),
//                   WasmParam(Value::I32(42)),
//                   Relative(TaskLabel("right"))
//               ]
//           }))
//       ])
//   }
