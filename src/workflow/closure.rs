//! The smallest unit of work in IPVM
use crate::workflow::pointer::{InvokedTaskPointer, Promise, Status};
use libipld::{
    cbor::DagCborCodec,
    cid::{multibase::Base, Version},
    prelude::Encode,
    Cid, Ipld, Link,
};
use multihash::{Code, MultihashDigest};
use std::{collections::btree_map::BTreeMap, result::Result};
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

impl Into<Link<Closure>> for Closure {
    fn into(self) -> Link<Closure> {
        let mut closure_bytes = Vec::new();
        <Closure as Into<Ipld>>::into(self).encode(DagCborCodec, &mut closure_bytes);

        Link::new(Cid::new_v1(
            DagCborCodec.into(),
            Code::Sha3_256.digest(&closure_bytes),
        ))
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

/// A newtype wrapper for [String]s.
///
/// The precise format is left open-ended, but by convention is namespaced with a single slash.
///
/// ```
/// use ipvm::workflow::closure::Action;
///
/// Action::from("msg/send");
/// Action::from("crud/update");
/// ```
///
/// Actions are case-insensitive, and don't respect wrapping whitespace:
///
/// ```
/// use ipvm::workflow::closure::Action;
///
/// let action = Action::from("eXaMpLe/tEsT".to_string());
/// let canonicalized: String = action.into();
///
/// assert_eq!(canonicalized, "example/test".to_string());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Action(String);

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action::from(s.to_string())
    }
}

impl From<String> for Action {
    fn from(s: String) -> Self {
        // Canonicalizes the wrapped string
        Action(s.trim().to_lowercase())
    }
}

impl Into<String> for Action {
    fn into(self) -> String {
        self.0
    }
}

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
