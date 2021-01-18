use std::collections::{BinaryHeap, VecDeque};

/// Error return when an attempt to push an element to a queue fails due to the queue having
/// reached its capacity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PushError;

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "queue reached its capacity")
    }
}

impl std::error::Error for PushError {}

/// Trait implemented by the queues used in the simulation.
pub trait Queue<T> {
    /// Add an element to the queue.
    ///
    /// # Errors
    ///
    /// Returns an error if the queue is bounded in size and full.
    fn push(&mut self, value: T) -> Result<(), PushError>;

    /// Removes the next element and returns it, or `None` if the `Queue` is empty.
    fn pop(&mut self) -> Option<T>;

    /// Returns the number of elements in the queue.
    fn len(&self) -> usize;

    /// Returns `true` if there are no elements in the queue.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Abstraction over [`VecDeque`] that allows to limit the capacity of the queue.
/// This means that push operations can fail.
/// By default, the capacity is equal to [`usize::MAX`], which makes unlimited in practice.
///
/// [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
/// [`usize::MAX`]: https://doc.rust-lang.org/std/primitive.usize.html#associatedconstant.MAX
/// ```
pub struct Fifo<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

impl<T> Default for Fifo<T> {
    fn default() -> Self {
        Self {
            inner: VecDeque::default(),
            capacity: usize::MAX,
        }
    }
}

impl<T> Fifo<T> {
    /// Creates a new queue with limited capacity.
    #[must_use]
    pub fn bounded(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
}

impl<T> Queue<T> for Fifo<T> {
    fn push(&mut self, value: T) -> Result<(), PushError> {
        if self.inner.len() < self.capacity {
            self.inner.push_back(value);
            Ok(())
        } else {
            Err(PushError)
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Binary heap implementation of [`Queue`].
pub struct PriorityQueue<T> {
    inner: BinaryHeap<T>,
    capacity: usize,
}

impl<T: Ord> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self {
            inner: BinaryHeap::default(),
            capacity: usize::MAX,
        }
    }
}

impl<T: Ord> PriorityQueue<T> {
    /// Creates a new queue with limited capacity.
    #[must_use]
    pub fn bounded(capacity: usize) -> Self {
        Self {
            inner: BinaryHeap::with_capacity(capacity),
            capacity,
        }
    }
}

impl<T: Ord> Queue<T> for PriorityQueue<T> {
    fn push(&mut self, value: T) -> Result<(), PushError> {
        if self.inner.len() < self.capacity {
            self.inner.push(value);
            Ok(())
        } else {
            Err(PushError)
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unbounded_queue() {
        let mut queue = Fifo::<i32>::default();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert!(queue.push(0).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
        assert!(queue.push(1).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(0));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_bounded_queue() {
        let mut queue = Fifo::<i32>::bounded(2);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert!(queue.push(0).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
        assert!(queue.push(1).is_ok());
        assert_eq!(queue.len(), 2);
        let err = queue.push(2).err();
        assert!(err.is_some());
        let err = err.unwrap();
        assert_eq!(&format!("{}", err), "queue reached its capacity");
        assert_eq!(queue.pop(), Some(0));
        assert_eq!(queue.len(), 1);
        assert!(queue.push(2).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_priority_queue() -> Result<(), PushError> {
        let queue = PriorityQueue::<i32>::default();
        assert_eq!(queue.capacity, usize::MAX);
        let mut queue = PriorityQueue::<i32>::bounded(2);
        assert_eq!(queue.capacity, 2);

        assert_eq!(queue.len(), 0);
        queue.push(1)?;
        assert_eq!(queue.len(), 1);
        queue.push(2)?;
        assert_eq!(queue.len(), 2);

        assert_eq!(queue.push(2).err(), Some(PushError));

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.len(), 0);

        Ok(())
    }
}
