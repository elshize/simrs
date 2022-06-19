use std::any::Any;
use std::collections::HashMap;

use super::{queue::PushError, Key, Queue, QueueId};

/// State of a simulation holding all queues and arbitrary values in a store value.
#[derive(Default)]
pub struct State {
    store: HashMap<usize, Box<dyn Any>>,
    queues: HashMap<usize, Box<dyn Any>>,
    next_id: usize,
}

#[allow(clippy::len_without_is_empty)]
impl State {
    /// Inserts an arbitrary value to the value store. Learn more in the documentation for [`Key`].
    #[must_use = "Discarding key results in leaking inserted value"]
    pub fn insert<V: 'static>(&mut self, value: V) -> Key<V> {
        let id = super::generate_next_id();
        self.store.insert(id, Box::new(value));
        Key::new(id)
    }

    /// Removes a value of type `V` from the value store. Learn more in the documentation for [`Key`].
    pub fn remove<V: 'static>(&mut self, key: Key<V>) -> Option<V> {
        self.store
            .remove(&key.id)
            .map(|v| *v.downcast::<V>().expect("Ensured by the key type."))
    }

    /// Gets a immutable reference to a value of a type `V` from the value store.
    /// Learn more in the documentation for [`Key`].
    #[must_use]
    pub fn get<V: 'static>(&self, key: Key<V>) -> Option<&V> {
        self.store
            .get(&key.id)
            .map(|v| v.downcast_ref::<V>().expect("Ensured by the key type."))
    }

    /// Gets a mutable reference to a value of a type `V` from the value store.
    /// Learn more in the documentation for [`Key`].
    #[must_use]
    pub fn get_mut<V: 'static>(&mut self, key: Key<V>) -> Option<&mut V> {
        self.store
            .get_mut(&key.id)
            .map(|v| v.downcast_mut::<V>().expect("Ensured by the key type."))
    }

    /// Creates a new unbounded queue, returning its ID.
    pub fn add_queue<Q: Queue + 'static>(&mut self, queue: Q) -> QueueId<Q> {
        let id = self.next_id;
        self.next_id += 1;
        self.queues.insert(id, Box::new(queue));
        QueueId::new(id)
    }

    /// Sends `value` to the `queue`. This is a shorthand for `queue_mut(queue).push(value)`.
    ///
    /// # Errors
    /// It returns an error if the queue is full.
    pub fn send<Q: Queue + 'static>(
        &mut self,
        queue: QueueId<Q>,
        value: Q::Item,
    ) -> Result<(), PushError> {
        self.queue_mut(queue).push(value)
    }

    /// Pops the first value from the `queue`. It returns `None` if  the queue is empty.
    /// This is a shorthand for `queue_mut(queue).pop(value)`.
    pub fn recv<Q: Queue + 'static>(&mut self, queue: QueueId<Q>) -> Option<Q::Item> {
        self.queue_mut(queue).pop()
    }

    /// Checks the number of elements in the queue.
    /// This is a shorthand for `queue(queue).len()`.
    #[must_use]
    pub fn len<Q: Queue + 'static>(&self, queue: QueueId<Q>) -> usize {
        self.queue(queue).len()
    }

    /// Returns a immutable reference to the queue by the given ID.
    #[must_use]
    pub fn queue<Q: Queue + 'static>(&self, queue: QueueId<Q>) -> &Q {
        self.queues
            .get(&queue.id)
            .expect("Queues cannot be removed so it must exist.")
            .downcast_ref::<Q>()
            .expect("Ensured by the key type.")
    }

    /// Returns a mutable reference to the queue by the given ID.
    #[must_use]
    pub fn queue_mut<Q: Queue + 'static>(&mut self, queue: QueueId<Q>) -> &mut Q {
        self.queues
            .get_mut(&queue.id)
            .expect("Queues cannot be removed so it must exist.")
            .downcast_mut::<Q>()
            .expect("Ensured by the key type.")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Fifo, PriorityQueue};

    #[test]
    fn test_add_remove_key_values() {
        let mut state = State::default();

        let id = state.insert(1);
        assert_eq!(state.remove(id), Some(1));
        assert_eq!(state.remove(id), None);

        let id = state.insert("string_slice");
        assert_eq!(state.get(id).copied(), Some("string_slice"));
        assert_eq!(state.remove(id), Some("string_slice"));
        assert_eq!(state.remove(id), None);

        let id = state.insert(vec![String::from("S")]);
        assert_eq!(state.remove(id), Some(vec![String::from("S")]));
        assert_eq!(state.remove(id), None);
    }

    #[test]
    fn test_modify_key_values() {
        let mut state = State::default();

        let id = state.insert(1);
        *state.get_mut(id).unwrap() = 2;
        assert_eq!(state.remove(id), Some(2));
        assert_eq!(state.remove(id), None);
    }

    #[test]
    fn test_bounded_queue() {
        let mut state = State::default();
        let qid = state.add_queue(Fifo::<&str>::bounded(2));
        assert_eq!(state.len(qid), 0);

        assert!(state.send(qid, "A").is_ok());
        assert!(state.send(qid, "B").is_ok());
        assert!(state.send(qid, "C").is_err());

        assert_eq!(state.recv(qid), Some("A"));
        assert_eq!(state.recv(qid), Some("B"));
        assert_eq!(state.recv(qid), None);
    }

    #[test]
    fn test_unbounded_queue() {
        let mut state = State::default();
        let qid = state.add_queue(Fifo::default());
        assert_eq!(state.len(qid), 0);

        assert!(state.send(qid, "A").is_ok());
        assert!(state.queue_mut(qid).push("B").is_ok());
        assert!(state.send(qid, "C").is_ok());

        assert_eq!(state.recv(qid), Some("A"));
        assert_eq!(state.recv(qid), Some("B"));
        assert_eq!(state.recv(qid), Some("C"));
        assert_eq!(state.recv(qid), None);
    }

    #[test]
    fn test_bounded_queue_priority() {
        let mut state = State::default();
        let qid = state.add_queue(PriorityQueue::bounded(2));
        assert_eq!(state.queue(qid).len(), 0);

        assert!(state.send(qid, 2).is_ok());
        assert!(state.send(qid, 1).is_ok());
        assert!(state.send(qid, 3).is_err());

        assert_eq!(state.recv(qid), Some(2));
        assert_eq!(state.recv(qid), Some(1));
        assert_eq!(state.recv(qid), None);
    }

    #[test]
    fn test_unbounded_queue_priority() {
        let mut state = State::default();
        let qid = state.add_queue(PriorityQueue::default());
        assert_eq!(state.len(qid), 0);

        assert!(state.send(qid, 2).is_ok());
        assert!(state.send(qid, 1).is_ok());
        assert!(state.send(qid, 3).is_ok());

        assert_eq!(state.recv(qid), Some(3));
        assert_eq!(state.recv(qid), Some(2));
        assert_eq!(state.recv(qid), Some(1));
        assert_eq!(state.recv(qid), None);
    }
}
