// use crate::metrics::Metrics;
// use crate::ur::{Ur, UrBuilder};

// pub struct AurBuilder<'a, T, S> {
//     builder: UrBuilder<'a, T, S>,
// }

// impl<'a> AurBuilder<'a> {
//     pub fn new() -> Self {
//         Self {
//             builder: UrBuilder::new(),
//         }
//     }

//     pub fn snapshot_trigger<F>(self, f: F) -> Self
//     where
//         F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
//     {
//         Self {
//             builder: self.builder.snapshot_trigger(f),
//         }
//     }

//     pub fn build<T: Clone>(self, initial_state: T) -> Aur<'a, T> {
//         Aur::new(self.builder.build(initial_state))
//     }
// }

// /// Ur<T> + Send + Sync
// pub struct Aur<'a, T>(Ur<'a, T>);

// impl<'a, T: Clone> Aur<'a, T> {
//     fn new(ur: Ur<'a, T>) -> Self {
//         Self(ur)
//     }
//     pub fn undo(&mut self) -> Option<&T> {
//         self.0.undo()
//     }
//     pub fn undo_multi(&mut self, count: usize) -> Option<&T> {
//         self.0.undo_multi(count)
//     }
//     pub fn redo(&mut self) -> Option<&T> {
//         self.0.redo()
//     }
//     pub fn redo_multi(&mut self, count: usize) -> Option<&T> {
//         self.0.redo_multi(count)
//     }
//     pub fn jumpdo(&mut self, count: isize) -> Option<&T> {
//         self.0.jumpdo(count)
//     }

//     pub fn edit<F>(&mut self, command: F) -> &T
//     where
//         F: Fn(T) -> T + Send + Sync + 'a,
//     {
//         self.0.edit(command)
//     }

//     pub fn edit_if<F>(&mut self, command: F) -> Option<&T>
//     where
//         F: Fn(T) -> Option<T> + Send + Sync + 'a,
//     {
//         self.0.edit_if(command)
//     }

//     pub fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
//     where
//         F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
//     {
//         self.0.try_edit(command)
//     }
// }

// // NOTE
// // Implementing the Send and Sync for Aur<T> is safe,
// // since Aur<T> guarantees that all of command and trigger functions implement the traits.
// unsafe impl<'a, T: Send> Send for Aur<'a, T> {}
// unsafe impl<'a, T: Sync> Sync for Aur<'a, T> {}

// impl<'a, T: std::fmt::Debug> std::fmt::Debug for Aur<'a, T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         self.0.fmt(f)
//     }
// }

// impl<'a, T: std::fmt::Display> std::fmt::Display for Aur<'a, T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         self.0.fmt(f)
//     }
// }

// impl<'a, T> std::ops::Deref for Aur<'a, T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         self.0.deref()
//     }
// }
