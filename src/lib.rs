//! Example
//!
//! ```rust
//! use gur::gur::{GurBuilder, Gur};
//!
//! fn main() {
//!     let mut gur = GurBuilder::new().build(0 as i32);
//!
//!     gur.edit(|n| n + 1);
//!     assert_eq!(1, *gur);
//!
//!     gur.undo().unwrap();
//!     assert_eq!(0, *gur);
//!
//!     gur.redo().unwrap();
//!     assert_eq!(1, *gur);
//! }
//! ```
pub mod gur;
pub mod metrics;
mod node;
