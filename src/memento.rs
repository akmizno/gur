/// Store and restore object's state.
///
/// [`Gur<T>`](crate::gur::Gur) requires T implementing [`Memento`] to take a snapshot of its internal state.
///
/// A simplest way to implement this trait is by cloning itself.
/// ```
/// # use gur::memento::Memento;
///
/// #[derive(Clone)]
/// struct MyState(String);
///
/// impl Memento for MyState {
///     type Target = Self;
///     fn to_memento(&self) -> Self::Target {
///         self.clone()
///     }
///     fn from_memento(memento: &Self::Target) -> Self {
///         memento.clone()
///     }
/// }
/// ```
pub trait Memento {
    type Target;
    fn to_memento(&self) -> Self::Target;
    fn from_memento(memento: &Self::Target) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct StringState(String);

    impl Memento for StringState {
        type Target = Self;
        fn to_memento(&self) -> Self::Target {
            self.clone()
        }
        fn from_memento(memento: &Self::Target) -> Self {
            memento.clone()
        }
    }

    #[test]
    fn string_as_string() {
        let state = StringState("Hello".to_string());

        let memento = state.to_memento();

        let restored = StringState::from_memento(&memento);

        assert_eq!(state, restored);
    }

    use std::collections::HashMap;

    #[derive(PartialEq, Debug)]
    struct MapState(HashMap<String, u32>);

    impl Memento for MapState {
        type Target = Vec<(String, u32)>;
        fn to_memento(&self) -> Self::Target {
            self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        }
        fn from_memento(memento: &Self::Target) -> Self {
            Self(memento.iter().map(|kv| kv.clone()).collect())
        }
    }

    #[test]
    fn hashmap_as_vec() {
        let mut map = HashMap::new();

        map.insert("foo".to_string(), 0);
        map.insert("bar".to_string(), 1);

        let state = MapState(map);

        let memento = state.to_memento();

        let restored = MapState::from_memento(&memento);

        assert_eq!(state, restored);
    }
}
