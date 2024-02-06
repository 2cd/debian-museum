use log::{debug, trace};
use serde::{Deserialize, Serialize};
use tinyvec::TinyVec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum MainRepo {
    Reg(String),
    Ghcr(String),
}

impl MainRepo {
    /// Returns `true` if the main repo is [`Ghcr`].
    ///
    /// [`Ghcr`]: MainRepo::Ghcr
    #[cfg(debug_assertions)]
    pub(crate) fn is_ghcr(&self) -> bool {
        matches!(self, Self::Ghcr(..))
    }
}

impl Default for MainRepo {
    fn default() -> Self {
        Self::Ghcr(Default::default())
    }
}

pub(crate) type Repos = TinyVec<[String; 16]>;

#[derive(Debug, Default, derive_more::Deref, Serialize, Deserialize)]
pub(crate) struct RepoMap(ahash::HashMap<MainRepo, Repos>);

impl RepoMap {
    /// Instead of resetting to a new value, this function pushes a new element to the value(&mut TinyVec) corresponding to the key.
    ///
    /// Note: If the corresponding key does not exist in the map, the Key and Value are created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut map = RepoMap::default();
    ///
    /// let key = MainRepo::Ghcr("ghcr.io/xx/yy:latest".into());
    ///
    /// map.push_to_value(key.to_owned(), "ghcr.io/xx/yy:x64".into());
    /// map.push_to_value(key.to_owned(), "ghcr.io/xx/yy:rv64gc".into());
    ///
    /// let value = map
    ///     .get(&key)
    ///     .expect("Failed to unwrap map");
    ///
    /// assert_eq!(value[0], "ghcr.io/xx/yy:x64");
    /// assert_eq!(value[1], "ghcr.io/xx/yy:rv64gc");
    /// ```
    pub(crate) fn push_to_value(&mut self, key: MainRepo, element: String) {
        self.0
            .entry(key)
            .and_modify(|v| {
                debug!("RepoMap.value\t is_heap: {}", v.is_heap());
                trace!("capacity: {}, len: {}", v.capacity(), v.len(),);
                v.push(element.clone())
            })
            .or_insert_with(|| {
                trace!("init RepoMap.value");
                let mut v = Repos::new();
                v.push(element);
                v
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_map() {
        let mut map = RepoMap::default();

        let key = MainRepo::Ghcr("ghcr.io/xx/yy:latest".into());

        map.push_to_value(key.clone(), "ghcr.io/xx/yy:x64".into());
        map.push_to_value(key.clone(), "ghcr.io/xx/yy:rv64gc".into());

        let value = map
            .get(&key)
            .expect("Failed to unwrap map");

        assert_eq!(value[0], "ghcr.io/xx/yy:x64");
        assert_eq!(value[1], "ghcr.io/xx/yy:rv64gc");
    }
}
