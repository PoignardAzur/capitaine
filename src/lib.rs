//! A prototype implementation of reactive UI in rust
//!
//! This project is a prototype implementation of a virtual-DOM-based UI for the Rust programming
//! language. It's inspired by [Crochet](https://github.com/raphlinus/crochet/) (hence the name) and
//! [React](https://github.com/facebook/react), and implemented on top of
//! [druid](https://github.com/linebender/druid).

/*

**Capitaine is an experimental GUI framework for the Rust programming language.**

This framework is **data-driven and declarative**, drawing some inspiration from [React](https://github.com/facebook/react), and implemented on top of the [Druid](https://github.com/linebender/druid) toolkit.

It aims to use **simple, idiomatic Rust**: Capitaine doesn't use unsafe code, cells, mutexes, or DSL macros.

## Getting started

Here is our "hello world" example:

```rust
use capitaine::elements::{Button, ButtonPressed, Label};
use capitaine::{make_row, ElementTree, ElementTreeExt, NoEvent, RootHandler};

#[derive(Debug, Default, Clone, PartialEq)]
struct HelloBoxState {
    count: i32,
}

fn hello_box(state: &HelloBoxState, _props: ()) -> impl ElementTree<HelloBoxState, NoEvent> {
    make_row!(
        Button::new("Say hello").on::<ButtonPressed, _>(|state: &mut HelloBoxState, _| {
            println!("Hello world - {}", state.count);
            state.count += 1;
        }),
        Label::new(format!("Hello count: {}", state.count)),
    )
}

fn main() -> Result<(), druid::PlatformError> {
    let state = HelloBoxState { count: 0 };

    RootHandler::new(&hello_box, state)
        .with_tracing(true)
        .launch()
}
```

To analyze this example, let's define a few terms:

- A **Widget** is the fundamental unit of GUI; for instance, a text field and a label are both widgets. You've probably seen the term if you've used other GUI frameworks.
- An **Element** is a lightweight description of a Widget. In our example, [Button.new] and [Label.new] both return elements. The [make_row] macros take an arbittrary number of elements and returns a container element.
- A **Component** is a user-written function that returns a tree of elements (or, more accurately, an arbitrary element that may or may not contain other elements). In our example, `hello_box` is a component.

In Capitaine, you don't directly manipulate **widgets**; instead, you write **components** that return **elements**. The framework calls your components, gets a tree of elements, and build a matching widget tree for you. When some event changes the application state, the framework calls your components again, gets a new element tree, and edits the widget tree accordingly.

For information on how to write a component, see TODO.


*/

mod element_tree;
mod glue;
mod root_handler;
mod widget_sequence;

pub mod elements;
pub mod widgets;

pub use element_tree::{ElementTree, ElementTreeExt, NoEvent};

pub use root_handler::{RootHandler, RootWidget};

/// Traits and type used internally to compute the GUI
pub mod backend {
    // Note: These items are declared together in files, but are exported separately here
    // to have a clean separation in the documentation between the items required to write
    // a GUI and the items required to create a GUI element.

    pub use crate::glue::{Action, DruidAppData, GlobalEventCx, Id};

    pub use crate::element_tree::{ReconcileCtx, VirtualDom};

    pub use crate::widget_sequence::{FlexWidget, WidgetSequence};

    // TODO
    pub use crate::widgets;
}
