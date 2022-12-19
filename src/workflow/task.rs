//! A [Closure] wrapped with Configuration, metadata, and optional secret.

use crate::workflow::{closure::Closure, config::Resources};
use libipld::{serde::from_ipld, Ipld};
use std::collections::BTreeMap;

const RESOURCES_KEY: &str = "resources";
const META_KEY: &str = "meta";
const SECRETS_KEY: &str = "secret";

const WITH_KEY: &str = "with";
const DO_KEY: &str = "do";
const INPUTS_KEY: &str = "inputs";

/// A [Task] is a [Closure] subtype, enriching it with information beyond
/// the meaning of the program like resource limits and visibility.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Description of the computation to run
    pub closure: Closure,

    /// Resource limits, timeouts, etc.
    pub resources: Resources,

    /// User-writable metadata
    pub metadata: Ipld,

    /// Whether the results should be hidden from the network.
    ///
    /// In the case that a secret is set, then this [Task] overrides the
    /// secrecy of the preceeding scope.
    ///
    /// If unset, [Task] inherits the secrecy of its scope, e.g.:
    /// If any [crate::workflow::Promise] in the [Closure] references a secret
    /// [Task], this [Task] will default to secret as well.
    ///
    /// If the enclosing [crate::workflow::Invocation] is set to secret, then
    /// this [Task] is also set to secret, unless a [Task] in one of its
    /// [crate::workflow::Promise]s is secret.
    pub secret: Option<bool>,
}

impl From<Closure> for Task {
    fn from(closure: Closure) -> Self {
        Task {
            closure,
            metadata: Ipld::Null,
            resources: Resources::default(),
            secret: None,
        }
    }
}

impl From<Task> for Ipld {
    fn from(task: Task) -> Ipld {
        Ipld::Map(BTreeMap::from([
            (WITH_KEY.into(), Ipld::String(task.closure.resource.into())),
            (DO_KEY.into(), task.closure.action.into()),
            (INPUTS_KEY.into(), task.closure.inputs.into()),
            (RESOURCES_KEY.into(), task.resources.into()),
            (
                SECRETS_KEY.into(),
                task.secret.map(Ipld::Bool).unwrap_or(Ipld::Null),
            ),
        ]))
    }
}

impl TryFrom<&Ipld> for Task {
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl TryFrom<Ipld> for Task {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld.clone())?;

        let resources = map
            .get(RESOURCES_KEY)
            .map_or_else(|| Ok(Resources::default()), Resources::try_from)?;

        Ok(Task {
            closure: Closure::try_from(ipld)?,
            resources,
            metadata: map.get(META_KEY).unwrap_or(&Ipld::Null).to_owned(),
            secret: map
                .get(SECRETS_KEY)
                .map(|ipld| from_ipld(ipld.to_owned()).unwrap_or(false)),
        })
    }
}
