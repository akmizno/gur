//! Example
//!
//! ```rust
//! use gur::cur::{CurBuilder, Cur};
//!
//! fn main() {
//!     let mut ur = CurBuilder::default().build(0 as i32);
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
pub mod acur;
pub mod aur;
pub mod cur;
pub mod gur;
pub mod metrics;
mod node;
pub mod snapshot;
pub mod triggers;
pub mod ur;
