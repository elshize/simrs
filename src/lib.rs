#![doc(html_root_url = "https://docs.rs/simrs/0.1.0")]
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::default_trait_access)]

//! General purpose simulation library that provides the mechanisms such as: scheduler, state,
//! queues, etc.
//!
//! **NOTE**: This is all experimental right now.
//!
//! The key public types are [`State`], [`Scheduler`], [`Components`], and [`Simulation`].
//! These, along with user-defined simulation components (structs implementing the [`Component`] trait),
//! are the simulation building blocks.
//! Let's first explain each of the above, and then look at examples.
//!
//! # State
//!
//! A simulation must have the ability to mutate its state.
//! This functionality is fully delegated to the [`State`] struct.
//! It can store, remove, and modify values of arbitrary types `T: 'static`.
//! It also allows us to create queues that can be used to move data between components.
//!
//! ## Value Store
//!
//! [`State`] exposes several simple functions to insert, access, modify, and remove values.
//! Existing values are manipulated using special type-safe keys that are generated and
//! returned when inserting the values.
//!
//! ```
//! # use simrs::State;
//! let mut state = State::default();
//! let key = state.insert(7);
//! assert_eq!(state.remove(key), Some(7));
//! ```
//!
//! Note that the following will fail to compile because of incompatible types:
//!
//! ```compile_fail
//! # use simrs::State;
//! let mut state = State::default();
//! let int_key = state.insert(7);
//! let str_key = state.insert("str");
//! let v: i32 = state.get(str_key);
//! ```
//!
//! ## Queues
//!
//! Queues work very similar to storing values but have a different user-facing interface.
//! The access is also done through a key type. However, a different type [`QueueId`] is
//! used for clarity.
//!
//! ```
//! # use simrs::State;
//! let mut state = State::default();
//! let queue_id = state.new_queue();
//! state.send(queue_id, 1);
//! assert_eq!(state.len(queue_id), 1);
//! assert_eq!(state.recv(queue_id), Some(1));
//! assert_eq!(state.recv(queue_id), None);
//! ```
//!
//! Additionally, a bounded queue is available, which will return an error if the size reached
//! the capacity.
//!
//! ```
//! # use simrs::State;
//! let mut state = State::default();
//! let queue_capacity = 1;
//! let queue_id = state.new_bounded_queue(queue_capacity);
//! assert!(state.send(queue_id, 1).is_ok());
//! assert_eq!(state.len(queue_id), 1);
//! assert!(!state.send(queue_id, 2).is_ok());
//! assert_eq!(state.len(queue_id), 1);
//! ```
//!
//! # Components
//!
//! The [`Components`] structure is a container for all registered components.
//! Similarly to values and queues in the state, components are identified by [`ComponentId`].
//!
//! ```
//! # use simrs::{Components, Component, State, Scheduler, ComponentId};
//! struct SomeComponent {
//!     // ...
//! }
//! #[derive(Debug)]
//! enum SomeEvent {
//!     A,
//!     B,
//!     C,
//! }
//! # impl SomeComponent {
//! #     fn new() -> Self {
//! #         SomeComponent {}
//! #     }
//! # }
//! impl Component for SomeComponent {
//!     type Event = SomeEvent;
//!     fn process_event(
//!         &self,
//!         self_id: ComponentId<Self::Event>,
//!         event: &Self::Event,
//!         scheduler: &mut Scheduler,
//!         state: &mut State,
//!     ) {
//!         // Do some work...
//!     }
//! }
//!
//! # fn main() {
//! let mut components = Components::default();
//! let component_id = components.add_component(SomeComponent::new());
//! # }
//! ```
//!
//! # Scheduler
//!
//! The scheduler's main functionality is to keep track of the simulation time and
//! the future events. Events are scheduled to run on a specific component at a specified
//! time interval. Because the events are type-erased, it's up to the component to
//! downcast the event. To make it easy, each component gets a blanket implementation
//! of an internal trait that does that automatically. It is all encapsulated in the
//! `Components` container, as shown in the below example:
//!
//! ```
//! # use simrs::{Components, Component, State, Scheduler, ComponentId};
//! # use std::time::Duration;
//! # struct SomeComponent {
//! #     // ...
//! # }
//! # #[derive(Debug)]
//! # enum SomeEvent {
//! #     A,
//! #     B,
//! #     C,
//! # }
//! # impl SomeComponent {
//! #     fn new() -> Self {
//! #         SomeComponent {}
//! #     }
//! # }
//! # impl Component for SomeComponent {
//! #     type Event = SomeEvent;
//! #     fn process_event(
//! #         &self,
//! #         self_id: ComponentId<Self::Event>,
//! #         event: &Self::Event,
//! #         scheduler: &mut Scheduler,
//! #         state: &mut State,
//! #     ) {
//! #         // Do some work...
//! #     }
//! # }
//! # fn main() {
//! let mut components = Components::default();
//! let mut scheduler = Scheduler::default();
//! let mut state = State::default();
//! let component_id = components.add_component(SomeComponent::new());
//! scheduler.schedule(
//!     Duration::from_secs(1), // schedule 1 second from now
//!     component_id,
//!     SomeEvent::A,
//! );
//! let event_entry = scheduler.pop().unwrap();
//! components.process_event_entry(event_entry, &mut scheduler, &mut state);
//! # }
//! ```
//!
//! # Simulation
//!
//! [`Simulation`] takes aggregates everything under one structure and provides some additional functions.
//! See the example below.
//!
//! # Example
//!
//! ```
//! # use simrs::{Simulation, State, Scheduler, Components, ComponentId, Component, QueueId, Key};
//! # use std::time::Duration;
//! struct Product;
//!
//! struct Producer {
//!     outgoing: QueueId<Product>,
//! }
//!
//! struct Consumer {
//!     incoming: QueueId<Product>,
//!     working_on: Key<Option<Product>>,
//! }
//!
//! #[derive(Debug)]
//! struct ProducerEvent;
//!
//! #[derive(Debug)]
//! enum ConsumerEvent {
//!     Received,
//!     Finished,
//! }
//!
//! impl Producer {
//!     fn produce(&self) -> Product { todo!() }
//!     fn interval(&self) -> std::time::Duration { todo!() }
//! }
//!
//! impl Consumer {
//!     fn produce(&self) -> Product { todo!() }
//!     fn interval(&self) -> std::time::Duration { todo!() }
//!     fn log(&self, product: Product) { todo!() }
//! }
//!
//! impl Component for Producer {
//!     type Event = ProducerEvent;
//!     
//!     fn process_event(
//!         &self,
//!         self_id: ComponentId<ProducerEvent>,
//!         _event: &ProducerEvent,
//!         scheduler: &mut Scheduler,
//!         state: &mut State,
//!     ) {
//!         state.send(self.outgoing, self.produce());
//!         scheduler.schedule(self.interval(), self_id, ProducerEvent);
//!     }
//! }
//!
//! impl Component for Consumer {
//!     type Event = ConsumerEvent;
//!     
//!     fn process_event(
//!         &self,
//!         self_id: ComponentId<ConsumerEvent>,
//!         event: &ConsumerEvent,
//!         scheduler: &mut Scheduler,
//!         state: &mut State,
//!     ) {
//!         let busy = state.get(self.working_on).is_none();
//!         match event {
//!             ConsumerEvent::Received => {
//!                 if busy {
//!                     if let Some(product) = state.recv(self.incoming) {
//!                         state
//!                             .get_mut(self.working_on)
//!                             .map(|w| *w = Some(product));
//!                         scheduler.schedule(
//!                             self.interval(),
//!                             self_id,
//!                             ConsumerEvent::Finished
//!                         );
//!                     }
//!                 }
//!             }
//!             ConsumerEvent::Finished => {
//!                 let product = state.get_mut(self.working_on).unwrap().take().unwrap();
//!                 self.log(product);
//!                 if state.len(self.incoming) > 0 {
//!                         scheduler.schedule(
//!                             Duration::default(),
//!                             self_id,
//!                             ConsumerEvent::Received
//!                         );
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

type Clock = Rc<Cell<Duration>>;

pub use component::{Component, Components};
pub use scheduler::{ClockRef, EventEntry, Scheduler};
pub use state::State;

use queue::Queue;

mod component;
mod queue;
mod scheduler;
mod state;

static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn generate_next_id() -> usize {
    ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Simulation struct that puts different parts of the simulation together.
///
/// See the [crate-level documentation](index.html) for more information.
pub struct Simulation {
    /// Simulation state.
    pub state: State,
    /// Event scheduler.
    pub scheduler: Scheduler,
    /// Component container.
    pub components: Components,
}

impl Simulation {
    /// Performs one step of the simulation. Returns `true` if there was in fact an event
    /// available to process, and `false` instead, which signifies that the simulation
    /// ended.
    pub fn step(&mut self) -> bool {
        self.scheduler.pop().map_or(false, |event| {
            self.components
                .process_event_entry(event, &mut self.scheduler, &mut self.state);
            true
        })
    }

    /// Runs the entire simulation from start to end.
    /// This function might not terminate if the end condition is not satisfied.
    pub fn run(&mut self) {
        while self.step() {}
    }

    /// Adds a new component.
    #[must_use]
    pub fn add_component<E: std::fmt::Debug + 'static, C: Component<Event = E> + 'static>(
        &mut self,
        component: C,
    ) -> ComponentId<E> {
        self.components.add_component(component)
    }

    /// Adds a new unbounded queue.
    #[must_use]
    pub fn add_queue<V: 'static>(&mut self) -> QueueId<V> {
        self.state.new_queue()
    }

    /// Adds a new bounded queue.
    #[must_use]
    pub fn add_bounded_queue<V: 'static>(&mut self, capacity: usize) -> QueueId<V> {
        self.state.new_bounded_queue(capacity)
    }

    /// Schedules a new event to be executed at time `time` in component `component`.
    pub fn schedule<E: std::fmt::Debug + 'static>(
        &mut self,
        time: Duration,
        component: ComponentId<E>,
        event: E,
    ) {
        self.scheduler.schedule(time, component, event);
    }
}

impl Default for Simulation {
    fn default() -> Self {
        let state = State::default();
        let components = Components::default();
        Self {
            state,
            components,
            scheduler: Scheduler::default(),
        }
    }
}

/// Defines a strongly typed key type.
macro_rules! key_type {
    ($name:ident, $inner:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $name<V> {
            pub(crate) id: $inner,
            _marker: PhantomData<V>,
        }
        impl<T> $name<T> {
            pub(crate) fn new(id: $inner) -> Self {
                $name {
                    id,
                    _marker: PhantomData,
                }
            }
        }
        impl<T> Clone for $name<T> {
            fn clone(&self) -> Self {
                Self::new(self.id)
            }
        }
        impl<T> Copy for $name<T> {}
    };
}

key_type!(
    ComponentId,
    usize,
    "A type-safe identifier of a component. This is an analogue of [`Key`] used specifically for components."
);

key_type!(
    Key,
    usize,
    r#"A type-safe key used to fetch values from the value store.

# Construction

A key can be constructed only by calling [`State::insert`].
The state assigns a new numerical ID to the inserted value, which is unique throughout
the running of the program.
This ensures type safety, as explained below.

# Type Safety

These keys are type-safe in a sense that a key used to insert a value of type `T` cannot be
used to access a value of another type `U`. An attempt to do so will result in a compile error.
It is achieved by having the key generic over `T`. However, `T` is just a marker, and no
values of type `T` are stored internally.

```compile_fail
# use simulation::{Key, State};
let mut state = State::default();
let id = state.insert(String::from("1"));
let _: Option<i32> = state.remove(id);  // Error!
let _ = state.remove::<i32>(id);        // Error!
```
"#
);

key_type!(
    QueueId,
    usize,
    r#"A type-safe identifier of a queue. This is an analogue of [`Key`] used specifically for queues."#
);
