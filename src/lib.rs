//! Example
//!
//! ```rust
//! use gur::gur::{GurBuilder, Gur};
//! use gur::snapshot::Snapshot;
//!
//! #[derive(Clone)]
//! struct MyState(i32);
//!
//! impl Snapshot for MyState {
//!     type Target = MyState;
//!     fn to_snapshot(&self) -> Self::Target {
//!         self.clone()
//!     }
//!     fn from_snapshot(s: &Self::Target) -> Self {
//!         s.clone()
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
pub mod metrics;
mod node;
pub mod snapshot;
