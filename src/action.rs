/// Modify or update a state to another state.
///
/// [`Gur<T>`](crate::gur::Gur) uses [`Action`] objects to update its internal state.
/// An [`Action`] object consumes a [`Gur<T>`](crate::gur::Gur)'s current state, then generates a next state.
///
/// Note that [`execute`](Action::execute) must produce same output even if it is invoked multiple times,
/// since undo/redo is implemented by regenerating [`Gur<T>`](crate::gur::Gur)'s internal states using the actions.
/// For actions cannot satisfy the constraint, use [`TryAction`].
///
/// Example
/// ```
/// # use gur::gur::{GurBuilder, Gur};
/// # use gur::action::Action;
/// struct MyState(i32);
///
/// struct Add(i32);
///
/// impl Action for Add {
///     type State = MyState;
///     fn execute(&self, prev: Self::State) -> Self::State {
///         MyState(prev.0 + self.0)
///     }
/// }
/// ```
pub trait Action {
    type State;
    fn execute(&self, prev: Self::State) -> Self::State;
}

/// Failable version of [`Action`].
///
/// Unlike [`Action`], [`try_execute`](TryAction::try_execute) is failable,
/// and it is guaranteed that the method is not invoked multiple times by [`Gur<T>`](crate::gur::Gur).
///
/// Example
/// ```
/// # use gur::action::TryAction;
/// # use std::io::Read;
/// struct MyState(i32);
///
/// struct FileAdd(std::path::PathBuf);
///
/// impl TryAction for FileAdd {
///     type State = MyState;
///     fn try_execute(&self, prev: Self::State) -> Result<Self::State, Box<dyn std::error::Error>> {
///         let mut file = std::fs::File::open(&self.0)?;
///         let mut s = String::new();
///         file.read_to_string(&mut s)?;
///         let n = s.parse::<i32>()?;
///         Ok(MyState(prev.0 + n))
///     }
/// }
/// ```
pub trait TryAction {
    type State;
    fn try_execute(&self, prev: Self::State) -> Result<Self::State, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Add(i32);

    impl Action for Add {
        type State = i32;
        fn execute(&self, prev: Self::State) -> Self::State {
            prev + self.0
        }
    }

    #[test]
    fn action_add() {
        let action = Add(1);

        let s = action.execute(0);

        assert_eq!(1, s);
    }

    #[derive(Debug)]
    struct Append(char);

    impl Action for Append {
        type State = String;
        fn execute(&self, mut prev: Self::State) -> Self::State {
            prev.push(self.0);
            prev
        }
    }

    #[test]
    fn action_append() {
        let action = Append('d');

        let s = action.execute("appen".to_string());

        assert_eq!("append", s);
    }

    #[derive(Debug)]
    struct OkAction(char);

    impl TryAction for OkAction {
        type State = String;
        fn try_execute(
            &self,
            mut prev: Self::State,
        ) -> Result<Self::State, Box<dyn std::error::Error>> {
            prev.push(self.0);
            Ok(prev)
        }
    }

    #[test]
    fn try_action_ok() {
        let action = OkAction('d');

        let s = action.try_execute("appen".to_string()).unwrap();

        assert_eq!("append", s);
    }

    #[derive(Debug)]
    struct ErrAction;

    impl TryAction for ErrAction {
        type State = i32;
        fn try_execute(&self, _: Self::State) -> Result<Self::State, Box<dyn std::error::Error>> {
            "NaN".parse::<i32>().map_err(|e| e.into())
        }
    }

    #[test]
    fn try_action_err() {
        let action = ErrAction;

        let s = action.try_execute(0);

        assert!(s.is_err());
    }
}
