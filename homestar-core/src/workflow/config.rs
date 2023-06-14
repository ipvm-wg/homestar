//! Homestar [workflow] configuration, typically expressed as metadata for
//! [Invocations].
//!
//! [workflow]: super
//! [Invocations]: super::Invocation

use crate::{workflow, Unit};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, default::Default, time::Duration};

const FUEL_KEY: &str = "fuel";
const TIMEOUT_KEY: &str = "time";

/// Resource configuration for defining fuel quota, timeout, etc.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Resources {
    fuel: Option<u64>,
    time: Option<Duration>,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            fuel: Some(u64::MAX),
            time: Some(Duration::from_millis(100_000)),
        }
    }
}

impl Resources {
    /// Create new [Resources] configuration.
    pub fn new(fuel: u64, time: Duration) -> Self {
        Self {
            fuel: Some(fuel),
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
}

impl From<Resources> for Ipld {
    fn from(resources: Resources) -> Ipld {
        Ipld::Map(BTreeMap::from([
            (
                FUEL_KEY.into(),
                resources.fuel().map(Ipld::from).unwrap_or(Ipld::Null),
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
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        Resources::try_from(ipld.to_owned())
    }
}

impl TryFrom<Ipld> for Resources {
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let fuel = map.get(FUEL_KEY).and_then(|ipld| match ipld {
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

        Ok(Resources { fuel, time })
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
                (TIMEOUT_KEY.into(), Ipld::Integer(100_000))
            ]))
        );
        assert_eq!(config, ipld.try_into().unwrap())
    }
}
