//! LinkMap type for storing generic values keyed by [Cid].

use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use libipld::Cid;

/// Generic link, cid => T [IndexMap] for storing
/// invoked, raw values in-memory and using them to
/// resolve other steps within a runtime's workflow.
///
/// [IndexMap]: IndexMap
#[derive(Debug, Clone, PartialEq)]
pub struct LinkMap<T>(IndexMap<Cid, T, FxBuildHasher>);

impl<T> Default for LinkMap<T> {
    fn default() -> Self {
        Self(IndexMap::with_hasher(FxBuildHasher::default()))
    }
}

impl<T> LinkMap<T> {
    /// Return a [LinkMap]'s [IndexMap].
    ///
    /// [IndexMap]: IndexMap
    pub fn take(self) -> IndexMap<Cid, T, FxBuildHasher> {
        self.0
    }

    /// Return a reference to [LinkMap]'s [IndexMap].
    ///
    /// [IndexMap]: IndexMap
    pub fn take_ref(&self) -> &IndexMap<Cid, T, FxBuildHasher> {
        &self.0
    }

    /// Length of [LinkMap]'s [IndexMap].
    ///
    /// [IndexMap]: IndexMap
    pub fn len(&self) -> u32 {
        self.0.len() as u32
    }

    /// Whether [LinkMap] contains [IndexMap] or not.
    ///
    /// [IndexMap]: IndexMap
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Insert a value into [LinkMap]'s [IndexMap].
    pub fn insert(&mut self, cid: Cid, value: T) {
        self.0.insert(cid, value);
    }

    /// Get a value from [LinkMap]'s [IndexMap].
    pub fn get(&self, cid: &Cid) -> Option<&T> {
        self.0.get(cid)
    }

    /// Return an entry to [LinkMap]'s [IndexMap].
    pub fn entry(&mut self, cid: Cid) -> indexmap::map::Entry<'_, Cid, T> {
        self.0.entry(cid)
    }

    /// Get a value from [LinkMap]'s [IndexMap].
    pub fn contains_key(&self, cid: &Cid) -> bool {
        self.0.contains_key(cid)
    }
}
