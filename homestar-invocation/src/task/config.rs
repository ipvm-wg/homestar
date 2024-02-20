//! [Invocation] configuration, typically expressed as metadata for tasks.
//!
//! [Invocation]: crate::Invocation

use crate::{consts, Error, Unit};
use libipld::{serde::from_ipld, Ipld};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

const FUEL_KEY: &str = "fuel";
const MEMORY_KEY: &str = "memory";
const TIMEOUT_KEY: &str = "time";

/// Resource configuration for defining fuel quota, timeout, etc.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[schemars(
    rename = "resources",
    description = "Resource configuration for fuel quota, memory allowance, and timeout"
)]
pub struct Resources {
    fuel: Option<u64>,
    #[schemars(description = "Memory in bytes")]
    memory: Option<u64>,
    #[schemars(with = "Option<u64>", description = "Timeout in milliseconds")]
    time: Option<Duration>,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            fuel: Some(u64::MAX),
            memory: Some(consts::WASM_MAX_MEMORY),
            time: Some(Duration::from_millis(100_000)),
        }
    }
}

impl Resources {
    /// Create new [Resources] configuration.
    pub fn new(fuel: u64, memory: u64, time: Duration) -> Self {
        Self {
            fuel: Some(fuel),
            memory: Some(memory),
            time: Some(time),
        }
    }

    /// Get fuel limit.
    pub fn fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Set fuel limit.
    pub fn set_fuel(&mut self, fuel: u64) {
        self.fuel = Some(fuel)
    }

    /// Get timeout.
    pub fn time(&self) -> Option<Duration> {
        self.time
    }

    /// Set timeout.
    pub fn set_time(&mut self, time: Duration) {
        self.time = Some(time)
    }

    /// Get max memory.
    pub fn memory(&self) -> Option<u64> {
        self.memory
    }

    /// Set max memory.
    pub fn set_memory(&mut self, memory: u64) {
        self.memory = Some(memory)
    }
}

impl From<Resources> for Ipld {
    fn from(resources: Resources) -> Ipld {
        Ipld::Map(BTreeMap::from([
            (
                FUEL_KEY.into(),
                resources.fuel().map(Ipld::from).unwrap_or(Ipld::Null),
            ),
            (
                MEMORY_KEY.into(),
                resources.memory().map(Ipld::from).unwrap_or(Ipld::Null),
            ),
            (
                TIMEOUT_KEY.into(),
                resources
                    .time()
                    .map(|t| Ipld::from(t.as_millis() as i128))
                    .unwrap_or(Ipld::Null),
            ),
        ]))
    }
}

impl<'a> TryFrom<&'a Ipld> for Resources {
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        Resources::try_from(ipld.to_owned())
    }
}

impl TryFrom<Ipld> for Resources {
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let fuel = map.get(FUEL_KEY).and_then(|ipld| match ipld {
            Ipld::Null => None,
            ipld => from_ipld(ipld.to_owned()).ok(),
        });

        let memory = map.get(MEMORY_KEY).and_then(|ipld| match ipld {
            Ipld::Null => None,
            ipld => from_ipld(ipld.to_owned()).ok(),
        });

        let time = map.get(TIMEOUT_KEY).and_then(|ipld| match ipld {
            Ipld::Null => None,
            ipld => {
                let time = from_ipld(ipld.to_owned()).unwrap_or(100_000);
                Some(Duration::from_millis(time))
            }
        });

        Ok(Resources { fuel, memory, time })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn ipld_roundtrip() {
        let config = Resources::default();
        let ipld = Ipld::from(config.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([
                (FUEL_KEY.into(), Ipld::Integer(u64::MAX.into())),
                (
                    MEMORY_KEY.into(),
                    Ipld::Integer(consts::WASM_MAX_MEMORY.into())
                ),
                (TIMEOUT_KEY.into(), Ipld::Integer(100_000))
            ]))
        );
        assert_eq!(config, ipld.try_into().unwrap())
    }

    #[test]
    fn ser_de() {
        let config = Resources::default();
        let ser = serde_json::to_string(&config).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(config, de);
    }
}
