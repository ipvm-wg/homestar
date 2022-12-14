//! The smallest unit of work in IPVM

use crate::workflow::pointer::{InvokedTaskPointer, Promise, Status};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec, cid::multibase::Base, prelude::Encode, serde as ipld_serde, Cid, Ipld, Link,
};
use multihash::{Code, MultihashDigest};
use std::{collections::btree_map::BTreeMap, convert::TryFrom, fmt};
use url::Url;

/// The suspended representation of the smallest unit of work in IPVM
///
/// ```
/// use libipld::Ipld;
/// use url::Url;
/// use ipvm::workflow::closure::{Closure, Action, Input};
///
/// Closure {
///     resource: Url::parse("ipfs://bafkreihf37goitzzlatlhwgiadb2wxkmn4k2edremzfjsm7qhnoxwlfstm").expect("IPFS URL"),
///     action: Action::from("wasm/run"),
///     inputs: Input::from(Ipld::Null),
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    /// The resource to be run.
    ///
    /// This may be any URL, including but not limited to
    /// `ipfs://`, `https://`, `mailto://`, and `data:`.
    pub resource: Url,

    /// The [Action] to be performed.
    pub action: Action,

    /// Some IPLD to pass to the action. The exact details will vary from action to action.
    pub inputs: Input,
}

impl TryInto<Link<Closure>> for Closure {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Link<Closure>, Self::Error> {
        let mut closure_bytes = Vec::new();
        <Closure as Into<Ipld>>::into(self).encode(DagCborCodec, &mut closure_bytes)?;
        Ok(Link::new(Cid::new_v1(
            DagCborCodec.into(),
            Code::Sha3_256.digest(&closure_bytes),
        )))
    }
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
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => Ok(Closure {
                action: Action::try_from(assoc.get("do").ok_or(anyhow!("Bad"))?.clone())
                    .or_else(|_| Err(anyhow!("Bad")))?,
                inputs: Input::from(assoc.get("inputs").ok_or(anyhow!("Bad"))?.clone()),
                resource: match assoc.get("with").ok_or(anyhow!("Bad"))? {
                    Ipld::Link(cid) => cid
                        .to_string_of_base(Base::Base32HexLower)
                        .or(Err(anyhow!("Bad")))
                        .and_then(|txt| {
                            Url::parse(format!("{}{}", "ipfs://", txt).as_str())
                                .or(Err(anyhow!("Bad")))
                        }),
                    Ipld::String(txt) => Url::parse(txt.as_str()).or(Err(anyhow!("Bad"))),
                    _ => Err(anyhow!("Bad")),
                }?,
            }),

            _ => Err(anyhow!("Bad")),
        }
    }
}

/// IPVM-flavoured inputs to a [Closure].
///
/// An `Input` is either [Ipld] or a deferred IPVM [Promise].
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    /// Literals
    IpldData(Ipld),

    /// Values from another Task
    Deferred(Promise),
}

impl Into<Ipld> for Input {
    fn into(self) -> Ipld {
        match self {
            Input::IpldData(ipld) => ipld,
            Input::Deferred(promise) => Promise::into(promise),
        }
    }
}

impl From<Promise> for Input {
    fn from(promise: Promise) -> Self {
        Input::Deferred(promise)
    }
}

impl From<Ipld> for Input {
    fn from(ipld: Ipld) -> Input {
        match ipld {
            Ipld::Map(ref map) => {
                if map.len() != 1 {
                    return Input::IpldData(ipld);
                }
                match map.get("ucan/ok") {
                    Some(Ipld::List(pointer)) => {
                        if let Ok(invoked_task) =
                            InvokedTaskPointer::try_from(Ipld::List(pointer.clone()))
                        {
                            Input::Deferred(Promise {
                                branch_selector: Some(Status::Success),
                                invoked_task,
                            })
                        } else {
                            Input::IpldData(ipld)
                        }
                    }

                    _ => Input::IpldData(ipld),
                }
            }
            _ => Input::IpldData(ipld),
        }
    }
}

/// A newtype wrapper for `do` Strings.
///
/// The precise format is left open-ended, but by convention is namespaced with
/// a single slash.
///
/// # Example
///
/// ```
/// use ipvm::workflow::closure::Action;
///
/// Action::from("msg/sen");
/// Action::from("crud/update");
/// ```
///
/// Actions are case-insensitive, and don't respect wrapping whitespace:
///
/// ```
/// use ipvm::workflow::closure::Action;
///
/// let action = Action::from("eXaMpLe/tEsT");
/// assert_eq!(action.to_string(), "example/test".to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action(String);

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a str> for Action {
    fn from(action: &'a str) -> Action {
        Action(action.trim().to_lowercase())
    }
}

impl From<String> for Action {
    fn from(action: String) -> Action {
        Action::from(action.as_str())
    }
}

impl From<Action> for Ipld {
    fn from(action: Action) -> Ipld {
        Ipld::String(action.0)
    }
}

impl TryFrom<Ipld> for Action {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let action = ipld_serde::from_ipld::<String>(ipld)?;
        Ok(Action::from(action))
    }
}
