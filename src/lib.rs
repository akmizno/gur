//! Example
//!
//! ```rust
//! use gur::ur::{UrBuilder, Ur};
//!
//! fn main() {
//!     let mut ur = UrBuilder::new().build(0 as i32);
//!
//!     ur.edit(|n| n + 1);
//!     assert_eq!(1, *ur);
//!
//!     ur.undo().unwrap();
//!     assert_eq!(0, *ur);
//!
//!     ur.redo().unwrap();
//!     assert_eq!(1, *ur);
//! }
//! ```
pub mod aur;
pub mod metrics;
mod node;
pub mod ur;
