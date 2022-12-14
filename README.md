# gur
A undo-redo framework.

# Generative approach
This section describes a basic concept of this crate.
The concept is that "Undoing is regenerating (recomputing) the old state."

For explanation, a sample history of changes is shown as follows,
```txt
t: state
c: command
s: snapshot

old <----------------------> new
     c1        c2        c3
t0 -----> t1 -----> t2 -----> t3
|  +-------------->           |
|  |                          |
s0 +--------------------------+
           undo t3 -> t2
```
Where `tx` is an application state at time point `x`,
`cx` is a command to change a state `tx-1` to next `tx`,
and `sx` is a snapshot of a state `tx`.

The application state have changed in order of `t0`, `t1`, `t2`, and `t3`.
Now, the current state is `t3`.

Let us consider undoing the current state `t3` to its previous state `t2`.
First, the system restores an old state from its snapshot at any point in the history.
In this case, We would have to restore the state `t0` from `s0` because there is only one snapshot `s0`.
Then the system reruns the commands (`c1` and `c2`) in order.
Finally, the target state `t2` will be obtained.

# Sample code
```rust
use gur::prelude::*;
use gur::cur::{Cur, CurBuilder};

// Appication state
#[derive(Clone)]
struct MyState {
    data: String
}

fn main() {
    // Initialize
    let mut state: Cur<MyState> = CurBuilder::new().build(MyState{ data: "My".to_string() });
    assert_eq!("My", state.data);

    // Change state
    state.edit(|mut state: MyState| { state.data += "State"; state });
    assert_eq!("MyState", state.data);

    // Undo
    state.undo();
    assert_eq!("My", state.data);

    // Redo
    state.redo();
    assert_eq!("MyState", state.data);
}
```
Where `Cur<T>` is a type providing undo-redo functionality.
The `MyState` is a type of user's application state.
`MyState` implements the `Clone` trait required by `Cur<T>`.
Then the variable `state` as type `Cur<MyState>` is created to get the ability to undo-redo.

The `edit` takes a closure to change the variable.
The closure is a function that consumes a current state as internal type `MyState` and returns a new state.
The `undo` can restore the previous state.
The `redo` is reverse operation of the `undo`.

The `Cur<T>` implements `Deref`.
So the variable can be used as like smart pointers.
