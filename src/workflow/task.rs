use crate::workflow::{closure::Closure, config::Resources};
use libipld::Ipld;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub closure: Closure,
    pub resources: Resources,
    pub metadata: Ipld,
    pub secret: Option<bool>,
}

impl Into<Ipld> for Task {
    fn into(self) -> Ipld {
        let secret_flag = match self.secret {
            None => Ipld::Null,
            Some(b) => Ipld::Bool(b),
        };

        Ipld::Map(BTreeMap::from([
            (
                "with".to_string(),
                Ipld::String(self.closure.resource.into()),
            ),
            ("do".to_string(), self.closure.action.into()),
            ("inputs".to_string(), self.closure.inputs.into()),
            ("resources".to_string(), self.resources.into()),
            ("secret".to_string(), secret_flag),
        ]))
    }
}

impl TryFrom<Ipld> for Task {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(ref assoc) => {
                let res = match assoc.get("resources") {
                    Some(v) => v.clone(),
                    _ => Ipld::Map(BTreeMap::new()),
                };

                let fuel: Option<u32> =
                    res.get("fuel")
                        .map_err(|_| ())
                        .and_then(|ipld_fuel| match ipld_fuel {
                            Ipld::Integer(int) => Ok(u32::try_from(*int).ok()),
                            _ => Err(()),
                        })?;

                let time: Option<u32> =
                    res.get("time")
                        .map_err(|_| ())
                        .and_then(|ipld_fuel| match ipld_fuel {
                            Ipld::Integer(int) => Ok(u32::try_from(*int).ok()),
                            _ => Err(()),
                        })?;

                let metadata: Ipld = match assoc.get("meta") {
                    Some(ipld) => ipld.clone(),
                    None => Ipld::Null,
                };

                // Is it secret? Is it safe?!
                let secret: Option<bool> =
                    assoc.get("secret").ok_or(()).and_then(|ipld| match ipld {
                        Ipld::Bool(b) => Ok(Some(*b)),
                        Ipld::Null => Ok(None),
                        _ => Err(()),
                    })?;

                Ok(Task {
                    closure: Closure::try_from(ipld)?,
                    resources: Resources { time, fuel },
                    metadata,
                    secret,
                })
            }
            _ => Err(()),
        }
    }
}
