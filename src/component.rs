use std::collections::HashMap;
use std::fmt;

use crate::{generate_next_id, ComponentId, EventEntry, Scheduler, State};

pub trait ProcessEventEntry {
    fn process_event_entry(&self, entry: EventEntry, scheduler: &mut Scheduler, state: &mut State);
}

/// Interface of a simulation component.
pub trait Component: ProcessEventEntry {
    /// Type of event the component reacts to.
    type Event: fmt::Debug + 'static;

    /// Reacts to `event`. A component has access to the following elements of the simulation:
    /// - `self_id`: This is the ID of this component. This is used to schedule events to itself.
    ///              This is passed for convenience, as the ID is only known after the component
    ///              has been already constructed and passed to the simulation.
    /// - `event`: The occurring event.
    /// - `scheduler`: The scheduler used to access time and schedule new events.
    /// - `state`: The state is used to access queues and values in the value store.
    fn process_event(
        &self,
        self_id: ComponentId<Self::Event>,
        event: &Self::Event,
        scheduler: &mut Scheduler,
        state: &mut State,
    );
}

impl<E, C> ProcessEventEntry for C
where
    E: fmt::Debug + 'static,
    C: Component<Event = E>,
{
    fn process_event_entry(&self, entry: EventEntry, scheduler: &mut Scheduler, state: &mut State) {
        let entry = entry
            .downcast::<E>()
            .expect("Failed to downcast event entry.");
        self.process_event(entry.component_id, entry.event, scheduler, state);
    }
}

/// Container holding type-erased components.
pub struct Components {
    components: HashMap<usize, Box<dyn ::std::any::Any>>,
}

impl Default for Components {
    #[must_use]
    fn default() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

impl Components {
    /// Process the event on the component given by the event entry.
    pub fn process_event_entry(
        &self,
        entry: EventEntry,
        scheduler: &mut Scheduler,
        state: &mut State,
    ) {
        self.components
            .get(&entry.component_idx())
            .unwrap()
            .downcast_ref::<Box<dyn ProcessEventEntry>>()
            .expect("Failed to downcast component.")
            .process_event_entry(entry, scheduler, state);
    }

    /// Registers a new component and returns its ID.
    #[must_use]
    pub fn add_component<E: fmt::Debug + 'static, C: Component<Event = E> + 'static>(
        &mut self,
        component: C,
    ) -> ComponentId<E> {
        let id = generate_next_id();
        let component: Box<dyn ProcessEventEntry> = Box::new(component);
        self.components.insert(id, Box::new(component));
        ComponentId::new(id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestComponent(Rc<RefCell<String>>);

    impl Component for TestComponent {
        type Event = String;

        fn process_event(
            &self,
            _self_id: ComponentId<Self::Event>,
            event: &Self::Event,
            _scheduler: &mut Scheduler,
            _state: &mut State,
        ) {
            *self.0.borrow_mut() = event.clone();
        }
    }

    struct RcTestComponent(String);

    impl Component for Rc<RefCell<RcTestComponent>> {
        type Event = String;

        fn process_event(
            &self,
            _self_id: ComponentId<Self::Event>,
            event: &Self::Event,
            _scheduler: &mut Scheduler,
            _state: &mut State,
        ) {
            self.borrow_mut().0 = event.clone();
        }
    }

    #[test]
    fn test_add_and_get_component() {
        let mut scheduler = Scheduler::default();
        let mut state = State::default();
        let mut components = Components::default();
        assert_eq!(components.components.len(), 0);

        let text = Rc::new(RefCell::new(String::from("")));

        let comp: ComponentId<String> = components.add_component(TestComponent(Rc::clone(&text)));
        assert_eq!(components.components.len(), 1);

        components.process_event_entry(
            EventEntry::new(
                std::time::Duration::default(),
                comp,
                String::from("Modified"),
            ),
            &mut scheduler,
            &mut state,
        );

        assert_eq!(*text.borrow(), "Modified");
    }

    #[test]
    fn test_rc_ref_cell() {
        let mut scheduler = Scheduler::default();
        let mut state = State::default();

        let component = Rc::new(RefCell::new(RcTestComponent(String::from(""))));
        let mut components = Components::default();
        let comp: ComponentId<String> = components.add_component(Rc::clone(&component));

        components.process_event_entry(
            EventEntry::new(
                std::time::Duration::default(),
                comp,
                String::from("Modified"),
            ),
            &mut scheduler,
            &mut state,
        );

        assert_eq!(component.borrow().0, "Modified");
    }
}
