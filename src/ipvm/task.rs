use cid::Cid;
use json::JsonValue;
use libipld::{cid::multibase::Base, Ipld, Link};
use signature::Signature;
use std::{collections::btree_map::BTreeMap, result::Result};
use ucan::ucan::Ucan;
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
pub struct Action(String);

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
    pub metadata: JsonValue,
    pub secret: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resources {
    pub fuel: Option<u32>,
    pub time: Option<u32>,
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

//impl TryFrom<Ipld> for Batch {
//    type Error = ();
//
//    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//        match ipld {
//            Ipld::Map(map) => {
//                let batch = BTreeMap::new();
//                for (key, value) in map.iter() {
//                  let label = TaskLabel::try_from(key)?;
//                  let task = Task::try_from(value)?;
//                  batch.insert(label, task);
//                }
//                Ok(Batch(batch))
//            },
//            _ => Err(())
//    }
//}

#[derive(Clone, Debug, PartialEq)]
pub struct Invocation<Sig: Signature> {
    pub run: Batch,
    pub sig: Sig,
    pub meta: Ipld,
    pub prf: Vec<Link<Ucan>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promise {
    pub invoked_task: InvokedTaskPointer,
    pub branch_selector: Option<Status>,
}

impl TryFrom<Ipld> for Promise {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(map) => {
                if map.len() != 1 {
                    return Err(());
                }

                let (key, value) = map.iter().next().unwrap();
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

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    IpldData { ipld: Ipld },
    Deferred { promise: Promise },
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskLabel(String);

impl TryFrom<Ipld> for TaskLabel {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(label) => Ok(TaskLabel(label)),
            _ => Err(()),
        }
    }
}

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
