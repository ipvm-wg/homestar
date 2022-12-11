/// This is the config module
use libipld::Ipld;
use std::collections::BTreeMap;

/// This is the config resources struct
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
