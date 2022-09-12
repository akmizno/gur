//! Example
//!
//! ```rust
//! use gur::gur::{GurBuilder, Gur};
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
//! fn main() {
//!     let mut gur = GurBuilder::new().build(MyState(0));
//!
//!     gur.edit(|state| MyState(state.0 + 1));
//!     assert_eq!(1, gur.0);
//!
//!     gur.undo().unwrap();
//!     assert_eq!(0, gur.0);
//!
//!     gur.redo().unwrap();
//!     assert_eq!(1, gur.0);
//! }
//! ```
pub mod gur;
pub mod memento;
pub mod metrics;
mod node;
