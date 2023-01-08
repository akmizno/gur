/// Undo-redo demo
/// This program checks which functions are called and how many times during `edit`, `undo`, and `redo`.
///
use gur::prelude::*;
use gur::snapshot::Snapshot;
use gur::triggers::snapshot_trigger::snapshot_by_distance;
use gur::ur::UrBuilder;

struct State(i32);

/// Snapshot implementation for State.
/// The methods print messages when they are called.
impl Snapshot for State {
    type Snapshot = i32;
    fn to_snapshot(&self) -> Self::Snapshot {
        println!("state {}: Take a snapshot.", self.0);
        self.0.clone()
    }
    fn from_snapshot(snapshot: &Self::Snapshot) -> Self {
        let state = snapshot.clone();
        println!("state {}: Restore from a snapshot({}).", state, snapshot);
        Self(state)
    }
}

/// A function to change the state.
/// A message is printed out when this function is called.
fn print_edit(current: State) -> State {
    let next = State(current.0 + 1);
    println!("state {} -> {}: Do command.", current.0, next.0);
    next
}

fn main() {
    println!("# INITIALIZE #");
    let mut state = UrBuilder::new()
        .snapshot_trigger(snapshot_by_distance(3))
        .build(State(0));

    println!("\n# EDIT 10 TIMES #");
    for i in 0..10 {
        println!("## Edit({}) ##", i);
        state.edit(|state| print_edit(state));
    }

    println!("\n# UNDO 10 TIMES #");
    for i in 0..10 {
        println!("## Undo({}) ##", i);
        state.undo();
    }

    println!("\n# REDO 10 TIMES #");
    for i in 0..10 {
        println!("## Redo({}) ##", i);
        state.redo();
    }
}
