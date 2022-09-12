//! Example
//!
//! ```rust
//! use gur::gur::{GurBuilder, Gur};
//! use gur::action::Action;
//! use gur::memento::Memento;
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
//!     let mut gur = GurBuilder::new().build(MyState(0));
//!
//!     gur.act(Add(1));
//!     assert_eq!(1, gur.0);
//!
//!     gur.undo().unwrap();
//!     assert_eq!(0, gur.0);
//!
//!     gur.redo().unwrap();
//!     assert_eq!(1, gur.0);
//! }
//! ```

pub mod action;
pub mod gur;
pub mod memento;
pub mod metrics;
mod node;
