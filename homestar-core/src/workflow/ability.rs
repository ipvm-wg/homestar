//! [UCAN Ability] for a given [Resource].
//!
//! [Resource]: url::Url
//! [UCAN Ability]: https://github.com/ucan-wg/spec/#23-ability

use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

/// A newtype wrapper for `call` fields, which contain a [UCAN Ability].
///
/// Abilities describe the verb portion of the capability: an ability that can
/// be performed on a resource. For instance, the standard HTTP methods such as
/// GET, PUT, and POST would be possible can values for an http resource.
///
/// The precise format is left open-ended, but by convention is namespaced with
/// a single slash.
///
/// # Example
///
/// ```
/// use homestar_core::workflow::Ability;
///
/// Ability::from("msg/send");
/// Ability::from("crud/update");
/// ```
///
/// Abilities are case-insensitive, and don't respect wrapping whitespace:
///
/// ```
/// use homestar_core::workflow::Ability;
///
/// let ability = Ability::from("eXaMpLe/tEsT");
/// assert_eq!(ability.to_string(), "example/test".to_string());
/// ```
///
/// [UCAN Ability]: https://github.com/ucan-wg/spec/#23-ability
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Ability(String);

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a str> for Ability {
    fn from(ability: &'a str) -> Ability {
        Ability(ability.trim().to_lowercase())
    }
}

impl From<String> for Ability {
    fn from(ability: String) -> Ability {
        Ability::from(ability.as_str())
    }
}

impl From<Ability> for Ipld {
    fn from(ability: Ability) -> Ipld {
        ability.0.into()
    }
}

impl TryFrom<Ipld> for Ability {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let ability = from_ipld::<String>(ipld)?;
        Ok(Ability::from(ability))
    }
}

impl<'a> From<Ability> for Cow<'a, Ability> {
    fn from(ability: Ability) -> Self {
        Cow::Owned(ability)
    }
}

impl<'a> From<&'a Ability> for Cow<'a, Ability> {
    fn from(ability: &'a Ability) -> Self {
        Cow::Borrowed(ability)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip() {
        let ability = Ability::from("wasm/run");
        let ipld = Ipld::from(ability.clone());

        assert_eq!(ipld, Ipld::String("wasm/run".to_string()));
        assert_eq!(ability, ipld.try_into().unwrap())
    }
}
