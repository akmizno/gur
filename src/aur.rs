use crate::metrics::Metrics;
use crate::ur::Ur;

pub struct AurBuilder<'a> {
    snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + Send + Sync + 'a>,
}

impl<'a> AurBuilder<'a> {
    pub fn new() -> Self {
        Self {
            snapshot_trigger: Box::new(|_m| false),
        }
    }

    pub fn snapshot_trigger<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Metrics) -> bool + Send + Sync + 'a,
    {
        self.snapshot_trigger = Box::new(f);
        self
    }

    pub fn build<T: Clone + 'a>(self, initial_state: T) -> Aur<'a, T> {
        Aur::new(initial_state, self.snapshot_trigger)
    }
}

/// Ur<T> + Send + Sync
pub struct Aur<'a, T>(Ur<'a, T>);

impl<'a, T: Clone + 'a> Aur<'a, T> {
    fn new(
        initial_state: T,
        snapshot_trigger: Box<dyn FnMut(&Metrics) -> bool + Send + Sync + 'a>,
    ) -> Self {
        Self(Ur::new(initial_state, snapshot_trigger))
    }
    pub fn undo(&mut self) -> Option<&T> {
        self.0.undo()
    }
    pub fn redo(&mut self) -> Option<&T> {
        self.0.redo()
    }

    pub fn edit<F>(&mut self, command: F) -> &T
    where
        F: Fn(T) -> T + Send + Sync + 'a,
    {
        self.0.edit(command)
    }

    pub fn try_edit<F>(&mut self, command: F) -> Result<&T, Box<dyn std::error::Error>>
    where
        F: FnOnce(T) -> Result<T, Box<dyn std::error::Error>>,
    {
        self.0.try_edit(command)
    }
}

// NOTE
// Implementing the Send and Sync for Aur<T> is safe,
// since Aur<T> guarantees that all of internally stored functions implement the traits.
unsafe impl<'a, T: Send> Send for Aur<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Aur<'a, T> {}

impl<'a, T: Default + Clone + 'a> Default for Aur<'a, T> {
    fn default() -> Self {
        AurBuilder::new().build(T::default())
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for Aur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T: std::fmt::Display> std::fmt::Display for Aur<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'a, T> std::ops::Deref for Aur<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
