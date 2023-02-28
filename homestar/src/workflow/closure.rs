//! The smallest unit of work in Homestar.

use crate::workflow::pointer::{InvokedTaskPointer, Promise, Status, OK_BRANCH};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::multibase::Base,
    codec::Codec,
    multihash::{Code, MultihashDigest},
    serde::from_ipld,
    Cid, Ipld, Link,
};
use std::{collections::btree_map::BTreeMap, convert::TryFrom, fmt};
use url::Url;

const WITH_KEY: &str = "with";
const DO_KEY: &str = "do";
const INPUTS_KEY: &str = "inputs";

/// The suspended representation of the smallest unit of work in Homestar.
///
/// # Example
///
/// ```
/// use libipld::Ipld;
/// use url::Url;
/// use homestar::workflow::closure::{Closure, Action, Input};
///
/// Closure {
///     resource: Url::parse("ipfs://bafkreihf37goitzzlatlhwgiadb2wxkmn4k2edremzfjsm7qhnoxwlfstm").expect("IPFS URL"),
///     action: Action::from("wasm/run"),
///     inputs: Input::try_from(Ipld::Null).unwrap(),
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

impl TryFrom<Closure> for Link<Closure> {
    type Error = anyhow::Error;

    fn try_from(closure: Closure) -> Result<Link<Closure>, Self::Error> {
        let ipld: Ipld = closure.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        Ok(Link::new(Cid::new_v1(
            DagCborCodec.into(),
            Code::Sha3_256.digest(&bytes),
        )))
    }
}

impl From<Closure> for Ipld {
    fn from(closure: Closure) -> Self {
        Ipld::Map(BTreeMap::from([
            (WITH_KEY.into(), Ipld::String(closure.resource.into())),
            (DO_KEY.into(), closure.action.into()),
            (INPUTS_KEY.into(), closure.inputs.into()),
        ]))
    }
}

impl TryFrom<Ipld> for Closure {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let action = Action::try_from(
            map.get(DO_KEY)
                .ok_or_else(|| anyhow!("no do action set."))?
                .to_owned(),
        )?;
        let inputs = Input::try_from(
            map.get(INPUTS_KEY)
                .ok_or_else(|| anyhow!("no inputs key set."))?
                .to_owned(),
        )?;

        let resource = match map.get(WITH_KEY) {
            Some(Ipld::Link(cid)) => cid
                .to_string_of_base(Base::Base32Lower)
                .map_err(|e| anyhow!("failed to encode CID into multibase string: {e}"))
                .and_then(|txt| {
                    Url::parse(format!("{}{}", "ipfs://", txt).as_str())
                        .map_err(|e| anyhow!("failed to parse URL: {e}"))
                }),
            Some(Ipld::String(txt)) => {
                Url::parse(txt.as_str()).map_err(|e| anyhow!("failed to parse URL: {e}"))
            }
            _ => Err(anyhow!("no resource/with set.")),
        }?;

        Ok(Closure {
            resource,
            action,
            inputs,
        })
    }
}

/// Homestar-flavoured inputs to a [Closure].
///
/// An `Input` is either [Ipld] or a deferred Homestar [Promise].
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    /// Literals
    IpldData(Ipld),

    /// Values from another Task
    Deferred(Promise),
}

impl From<Input> for Ipld {
    fn from(input: Input) -> Self {
        match input {
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

impl TryFrom<Ipld> for Input {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let Ok(map) = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned()) else {
            return Ok(Input::IpldData(ipld))
        };

        if map.len() > 1 {
            map.get(OK_BRANCH)
                .map_or(Ok(Input::IpldData(ipld)), |ipld| {
                    let pointer = from_ipld(ipld.to_owned())?;
                    let invoked_task = InvokedTaskPointer::try_from(Ipld::List(pointer))?;
                    Ok(Input::Deferred(Promise {
                        result: Some(Status::Success),
                        invoked_task,
                    }))
                })
        } else {
            Ok(Input::IpldData(ipld))
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
/// use homestar::workflow::closure::Action;
///
/// Action::from("msg/send");
/// Action::from("crud/update");
/// ```
///
/// Actions are case-insensitive, and don't respect wrapping whitespace:
///
/// ```
/// use homestar::workflow::closure::Action;
///
/// let action = Action::from("eXaMpLe/tEsT");
/// assert_eq!(action.to_string(), "example/test".to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action(String);

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        let action = from_ipld::<String>(ipld)?;
        Ok(Action::from(action))
    }
}
