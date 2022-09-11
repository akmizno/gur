//! Example
//! ```
//! use ur::ur::{UrBuilder, Ur};
//! use ur::action::Action;
//! use ur::memento::Memento;
//!
//! #[derive(Clone)]
//! struct MyState(i32);
//!
//! impl Memento for MyState {
//!     type Target = MyState;
//!     fn to_memento(&self) -> Self::Target {
//!         self.clone()
//!     }
//!     fn from_memento(m: &Self::Target) -> Self {
//!         m.clone()
//!     }
//! }
//!
//! struct Add(i32);
//!
//! impl Action for Add {
//!     type State = MyState;
//!     fn execute(&self, prev: Self::State) -> Self::State {
//!         MyState(prev.0 + self.0)
//!     }
//! }
//!
//! fn main() {
//!     let mut ur = UrBuilder::new().build(MyState(0));
//!
//!     ur.act(Add(1));
//!     assert_eq!(1, ur.0);
//!
//!     ur.undo().unwrap();
//!     assert_eq!(0, ur.0);
//!
//!     ur.redo().unwrap();
//!     assert_eq!(1, ur.0);
//! }
//! ```
pub mod action;
pub mod memento;
pub(crate) mod node;
pub mod ur;
