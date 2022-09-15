/// Store and restore object's state.
///
/// [`Gur<T>`](crate::gur::Gur) requires T implementing [`Snapshot`] to take a snapshot from its internal state.
///
/// A simplest way to implement this trait is by cloning itself.
/// ```
/// # use gur::snapshot::Snapshot;
///
/// #[derive(Clone)]
/// struct MyState(String);
///
/// impl Snapshot for MyState {
///     type Target = Self;
///     fn to_snapshot(&self) -> Self::Target {
///         self.clone()
///     }
///     fn from_snapshot(snapshot: &Self::Target) -> Self {
///         snapshot.clone()
///     }
/// }
/// ```
pub trait Snapshot {
    type Target;
    fn to_snapshot(&self) -> Self::Target;
    fn from_snapshot(snapshot: &Self::Target) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct StringState(String);

    impl Snapshot for StringState {
        type Target = Self;
        fn to_snapshot(&self) -> Self::Target {
            self.clone()
        }
        fn from_snapshot(snapshot: &Self::Target) -> Self {
            snapshot.clone()
        }
    }

    #[test]
    fn string_as_string() {
        let state = StringState("Hello".to_string());

        let snapshot = state.to_snapshot();

        let restored = StringState::from_snapshot(&snapshot);

        assert_eq!(state, restored);
    }

    use std::collections::HashMap;

    #[derive(PartialEq, Debug)]
    struct MapState(HashMap<String, u32>);

    impl Snapshot for MapState {
        type Target = Vec<(String, u32)>;
        fn to_snapshot(&self) -> Self::Target {
            self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        }
        fn from_snapshot(snapshot: &Self::Target) -> Self {
            Self(snapshot.iter().map(|kv| kv.clone()).collect())
        }
    }

    #[test]
    fn hashmap_as_vec() {
        let mut map = HashMap::new();

        map.insert("foo".to_string(), 0);
        map.insert("bar".to_string(), 1);

        let state = MapState(map);

        let snapshot = state.to_snapshot();

        let restored = MapState::from_snapshot(&snapshot);

        assert_eq!(state, restored);
    }
}
