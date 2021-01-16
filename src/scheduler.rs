use std::any::Any;
use std::cell::Cell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

use crate::{Clock, ComponentId};

/// Entry type stored in the scheduler, including the event value, component ID, and the time when
/// it is supposed to occur.
///
/// Besides being stored in the scheduler's internal priority queue,
/// event entries are simply passed to [`crate::Components`] object, which unpacks them, and passes them
/// to the correct component.
#[derive(Debug)]
pub struct EventEntry {
    time: Reverse<Duration>,
    component: usize,
    inner: Box<dyn Any>,
}

impl EventEntry {
    pub(crate) fn new<E: fmt::Debug + 'static>(
        time: Duration,
        component: ComponentId<E>,
        event: E,
    ) -> Self {
        EventEntry {
            time: Reverse(time),
            component: component.id,
            inner: Box::new(event),
        }
    }

    /// Tries to downcast the event entry to one holding an event of type `E`.
    /// If fails, returns `None`.
    #[must_use]
    pub(crate) fn downcast<E: fmt::Debug + 'static>(&self) -> Option<EventEntryTyped<'_, E>> {
        self.inner.downcast_ref::<E>().map(|event| EventEntryTyped {
            time: self.time.0,
            component_id: ComponentId::new(self.component),
            component_idx: self.component,
            event,
        })
    }

    #[must_use]
    pub(crate) fn component_idx(&self) -> usize {
        self.component
    }
}

impl PartialEq for EventEntry {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for EventEntry {}

impl PartialOrd for EventEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for EventEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Debug)]
pub struct EventEntryTyped<'e, E: fmt::Debug> {
    pub time: Duration,
    pub component_id: ComponentId<E>,
    pub component_idx: usize,
    pub event: &'e E,
}

/// This struct exposes only immutable access to the simulation clock.
/// The clock itself is owned by the scheduler, while others can obtain `ClockRef`
/// to read the current simulation time.
///
/// # Example
///
/// ```
/// # use simrs::Scheduler;
/// let scheduler = Scheduler::default();
/// let clock_ref = scheduler.clock();
/// assert_eq!(clock_ref.time(), scheduler.time());
/// ```
pub struct ClockRef {
    clock: Clock,
}

impl ClockRef {
    /// Return the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }
}

/// Scheduler is used to keep the current time and information about the upcoming events.
///
/// See the [crate-level documentation](index.html) for more information.
pub struct Scheduler {
    events: BinaryHeap<EventEntry>,
    clock: Clock,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            events: BinaryHeap::default(),
            clock: Rc::new(Cell::new(Duration::default())),
        }
    }
}

impl Scheduler {
    /// Schedules `event` to be executed for `component` at `self.time() + time`.
    pub fn schedule<E: fmt::Debug + 'static>(
        &mut self,
        time: Duration,
        component: ComponentId<E>,
        event: E,
    ) {
        let time = self.time() + time;
        self.events.push(EventEntry::new(time, component, event));
    }

    /// Schedules `event` to be executed for `component` at `self.time()`.
    pub fn schedule_now<E: fmt::Debug + 'static>(&mut self, component: ComponentId<E>, event: E) {
        self.schedule(Duration::default(), component, event);
    }

    /// Returns the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }

    /// Returns a structure with immutable access to the simulation time.
    #[must_use]
    pub fn clock(&self) -> ClockRef {
        ClockRef {
            clock: Rc::clone(&self.clock),
        }
    }

    /// Removes and returns the next scheduled event or `None` if none are left.
    pub fn pop(&mut self) -> Option<EventEntry> {
        self.events.pop().map(|event| {
            self.clock.replace(event.time.0);
            event
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_event_entry_debug() {
        let entry = EventEntry {
            time: Reverse(Duration::from_secs(1)),
            component: 2,
            inner: Box::new(String::from("inner")),
        };
        assert_eq!(
            &format!("{:?}", entry),
            "EventEntry { time: Reverse(1s), component: 2, inner: Any }"
        );
        let typed = entry.downcast::<String>().unwrap();
        assert_eq!(
            &format!("{:?}", typed),
            "EventEntryTyped { time: 1s, component_id: ComponentId { id: 2, _marker: PhantomData }, component_idx: 2, event: \"inner\" }"
        );
    }

    #[test]
    fn test_event_entry_downcast() {
        let entry = EventEntry {
            time: Reverse(Duration::from_secs(1)),
            component: 2,
            inner: Box::new(String::from("inner")),
        };
        assert!(entry.downcast::<String>().is_some());
        assert!(entry.downcast::<i32>().is_none());
    }

    #[test]
    fn test_event_entry_cmp() {
        let make_entry = || EventEntry {
            time: Reverse(Duration::from_secs(1)),
            component: 2,
            inner: Box::new(String::from("inner")),
        };
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            },
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(0)),
                ..make_entry()
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }),
            Ordering::Greater
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(2)),
                ..make_entry()
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }),
            Ordering::Less
        );
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct EventA;
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct EventB;

    #[test]
    fn test_scheduler() {
        let mut scheduler = Scheduler::default();
        assert_eq!(scheduler.time(), Duration::new(0, 0));
        assert_eq!(scheduler.clock().time(), Duration::new(0, 0));
        assert!(scheduler.events.is_empty());

        let component_a = ComponentId::<EventA>::new(0);
        let component_b = ComponentId::<EventB>::new(1);

        scheduler.schedule(Duration::from_secs(1), component_a, EventA);
        scheduler.schedule_now(component_b, EventB);
        scheduler.schedule(Duration::from_secs(2), component_b, EventB);

        assert_eq!(scheduler.time(), Duration::from_secs(0));

        let entry = scheduler.pop().unwrap();
        let entry = entry.downcast::<EventB>().unwrap();
        assert_eq!(entry.time, Duration::from_secs(0));
        assert_eq!(entry.component_idx, 1);
        assert_eq!(entry.component_id, component_b);
        assert_eq!(entry.event, &EventB);

        assert_eq!(scheduler.time(), Duration::from_secs(0));

        let entry = scheduler.pop().unwrap();
        let entry = entry.downcast::<EventA>().unwrap();
        assert_eq!(entry.time, Duration::from_secs(1));
        assert_eq!(entry.component_idx, 0);
        assert_eq!(entry.component_id, component_a);
        assert_eq!(entry.event, &EventA);

        assert_eq!(scheduler.time(), Duration::from_secs(1));
        assert_eq!(scheduler.clock().time(), Duration::from_secs(1));

        let entry = scheduler.pop().unwrap();
        let entry = entry.downcast::<EventB>().unwrap();
        assert_eq!(entry.time, Duration::from_secs(2));
        assert_eq!(entry.component_idx, 1);
        assert_eq!(entry.component_id, component_b);
        assert_eq!(entry.event, &EventB);

        assert_eq!(scheduler.time(), Duration::from_secs(2));

        assert!(scheduler.pop().is_none());
    }
}
