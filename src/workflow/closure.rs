use crate::workflow::pointer::{InvokedTaskPointer, Promise, Status};
use libipld::{cid::multibase::Base, Ipld};
use std::{collections::btree_map::BTreeMap, result::Result};
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub resource: Url,
    pub action: Action,
    pub inputs: Input,
}

impl Into<Ipld> for Closure {
    fn into(self) -> Ipld {
        Ipld::Map(BTreeMap::from([
            ("with".to_string(), Ipld::String(self.resource.into())),
            ("do".to_string(), self.action.into()),
            ("inputs".to_string(), self.inputs.into()),
        ]))
    }
}

impl TryFrom<Ipld> for Closure {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => Ok(Closure {
                action: Action::try_from(assoc.get("do").ok_or(())?.clone()).or(Err(()))?,
                inputs: Input::from(assoc.get("inputs").ok_or(())?.clone()),
                resource: match assoc.get("with").ok_or(())? {
                    Ipld::Link(cid) => match cid.to_string_of_base(Base::Base32HexLower) {
                        Ok(txt) => {
                            let ipfs_url: String = format!("{}{}", "ipfs://", txt);
                            Url::parse(ipfs_url.as_str()).or(Err(()))
                        }
                        _ => Err(()),
                    },
                    Ipld::String(txt) => Url::parse(txt.as_str()).or(Err(())),
                    _ => Err(()),
                }?,
            }),

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
pub struct Action(pub String);

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
