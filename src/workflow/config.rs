//! Configuration module

use libipld::{serde as ipld_serde, Ipld};
use std::{collections::BTreeMap, default::Default, time::Duration};

const FUEL_KEY: &str = "fuel";
const TIMEOUT_KEY: &str = "time";

/// IPVM resource configuration for defining fuel quotas, timeouts, etc.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resources {
    fuel: Option<u64>,
    time: Option<Duration>,
}

impl Resources {
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
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        Resources::try_from(ipld.to_owned())
    }
}

impl TryFrom<Ipld> for Resources {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let fuel = ipld_serde::from_ipld(ipld.get(FUEL_KEY)?.to_owned())?;
        let time = ipld_serde::from_ipld(ipld.take(TIMEOUT_KEY)?)?;

        Ok(Resources { fuel, time })
    }
}
