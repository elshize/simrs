use std::any::Any;
use std::collections::HashMap;

use super::{
    queue::{Fifo, PushError},
    Key, Queue, QueueId,
};

/// State of a simulation holding all queues and arbitrary values in a store value.
pub struct State {
    store: HashMap<usize, Box<dyn Any>>,
    queues: HashMap<usize, Box<dyn Any>>,
    next_id: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            store: HashMap::new(),
            queues: HashMap::new(),
            next_id: 0,
        }
    }
}

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
    pub fn add_queue<Q: Queue<V> + 'static, V: 'static>(&mut self, queue: Q) -> QueueId<V> {
        let id = self.next_id;
        self.next_id += 1;
        self.queues.insert(id, Box::new(queue));
        QueueId::new(id)
    }

    /// Sends `value` to the `queue`.
    ///
    /// # Errors
    /// It returns an error if the queue is full.
    pub fn send<V: 'static>(&mut self, queue: QueueId<V>, value: V) -> Result<(), PushError> {
        self.queues
            .get_mut(&queue.id)
            .expect("Queues cannot be removed so it must exist.")
            .downcast_mut::<Fifo<V>>()
            .expect("Ensured by the key type.")
            .push(value)
    }

    /// Pops the first value from the `queue`. It returns `None` if  the queue is empty.
    pub fn recv<V: 'static>(&mut self, queue: QueueId<V>) -> Option<V> {
        self.queues
            .get_mut(&queue.id)
            .expect("Queues cannot be removed so it must exist.")
            .downcast_mut::<Fifo<V>>()
            .expect("Ensured by the key type.")
            .pop()
    }

    /// Checks the number of elements in the queue.
    pub fn len<V: 'static>(&mut self, queue: QueueId<V>) -> usize {
        self.queues
            .get(&queue.id)
            .expect("Queues cannot be removed so it must exist.")
            .downcast_ref::<Fifo<V>>()
            .expect("Ensured by the key type.")
            .len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert!(state.send(qid, "B").is_ok());
        assert!(state.send(qid, "C").is_ok());

        assert_eq!(state.recv(qid), Some("A"));
        assert_eq!(state.recv(qid), Some("B"));
        assert_eq!(state.recv(qid), Some("C"));
        assert_eq!(state.recv(qid), None);
    }
}
