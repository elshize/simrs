use std::collections::VecDeque;

/// Abstraction over [`VecDeque`] that allows to limit the capacity of the queue.
/// This means that push operations can fail.
/// By default, the capacity is equal to [`usize::MAX`], which makes unlimited in practice.
///
/// [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
/// [`usize::MAX`]: https://doc.rust-lang.org/std/primitive.usize.html#associatedconstant.MAX
/// ```
pub struct Queue<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            inner: VecDeque::default(),
            capacity: usize::MAX,
        }
    }
}

impl<T> Queue<T> {
    /// Creates a queue with the given capacity.
    pub fn bounded(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Appends an element to the back of the `Queue`.
    pub fn push_back(&mut self, value: T) -> Result<(), ()> {
        if self.inner.len() < self.capacity {
            self.inner.push_back(value);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Removes the first element and returns it, or `None` if the `Queue` is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    /// Returns the number of elements in the `Queue`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unbounded_queue() {
        let mut queue = Queue::<i32>::default();
        assert_eq!(queue.len(), 0);
        assert!(queue.push_back(0).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(queue.push_back(1).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop_front(), Some(0));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop_front(), Some(1));
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop_front(), None);
    }

    #[test]
    fn test_bounded_queue() {
        let mut queue = Queue::<i32>::bounded(2);
        assert_eq!(queue.len(), 0);
        assert!(queue.push_back(0).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(queue.push_back(1).is_ok());
        assert_eq!(queue.len(), 2);
        assert!(queue.push_back(2).is_err());
        assert_eq!(queue.pop_front(), Some(0));
        assert_eq!(queue.len(), 1);
        assert!(queue.push_back(2).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop_front(), Some(1));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop_front(), Some(2));
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop_front(), None);
    }
}
