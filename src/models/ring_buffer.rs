use std::ops::{Index, IndexMut};

use crate::enums::order_book_errors::OrderBookError;

/**
 * 
 *  NOTE: N must be a power of 2 due to wrapping calculation.
 * 
 */

 #[derive(Clone)]
pub struct RingBuffer<const N: usize> {
    pub buf: [usize; N],
    pub head: usize,
    pub len: usize
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            len: 0
        }
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn push_back(&mut self, value: usize) -> Result<(), OrderBookError> {
        if self.is_full() {
            return Err(OrderBookError::FullRingBuffer);
        }
        
        let idx = (self.head + self.len) & (N - 1);
        self.buf[idx] = value;
        self.len += 1;

        Ok(())
    }

    #[inline(always)]
    pub fn push_front(&mut self, value: usize) -> Result<(), OrderBookError> {
        if self.is_full() {
            return Err(OrderBookError::FullRingBuffer);
        }

        self.head = (self.head + N - 1) & (N - 1);
        self.buf[self.head] = value;
        self.len += 1;

        Ok(())
    }

    #[inline(always)]
    pub fn pop_back(&mut self) -> Result<usize, OrderBookError> {
        if self.is_empty() {
            return Err(OrderBookError::EmptyRingBuffer);
        }

        let idx = (self.head + self.len - 1) % N;
        let val = self.buf[idx];
        self.len -= 1;

        Ok(val)
    }

    #[inline(always)]
    pub fn pop_front(&mut self) -> Result<usize, OrderBookError> {
        if self.is_empty() {
            return Err(OrderBookError::EmptyRingBuffer);
        }

        let v = self.buf[self.head];
        self.head = (self.head + 1) & (N - 1);
        self.len -= 1;

        Ok(v)
    }

    #[inline]
    pub fn front(&self) -> Option<usize> {
        if self.is_empty() {
            None
        }
        else {
            Some(self.buf[self.head])
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item=usize> + '_ {
        (0..self.len).map(move |i| self.buf[(self.head + i) & (N - 1)])
    }

    pub fn remove_by_value(&mut self, val: usize) -> bool {
        for i in 0..self.len {
            let idx = (self.head + i) & (N - 1);
            if self.buf[idx] == val {
                for j in i..(self.len - 1) {
                    let from = (self.head + j + 1) & (N - 1);
                    let to = (self.head + j) & (N - 1);
                    self.buf[to] = self.buf[from];
                }

                self.len -= 1;
                return true;
            }
        }
        return false;
    }
}

impl<const N: usize> Index<usize> for RingBuffer<N> {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}

impl<const N: usize> IndexMut<usize> for RingBuffer<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            len: 0
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_empty_correctly_returns_true_for_empty_ring_buffer() {
        let ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert!(ring_buffer.is_empty());
    }

    #[test]
    fn test_is_empty_correctly_returns_false_for_ring_buffer_with_elements() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        let idx = (ring_buffer.head + ring_buffer.len) & (128 - 1);
        ring_buffer[idx] = 5;
        ring_buffer.len += 1;

        assert!(!ring_buffer.is_empty());
    }

    #[test]
    fn test_is_full_correctly_returns_true_for_full_ring_buffer() {
        let mut ring_buffer: RingBuffer<1> = RingBuffer::new();

        let idx = (ring_buffer.head + ring_buffer.len) & (128 - 1);
        ring_buffer[idx] = 5;
        ring_buffer.len += 1;

        assert!(ring_buffer.is_full());
    }

    #[test]
    fn test_is_full_correctly_returns_false_for_ring_buffer_with_free_slots() {
        let ring_buffer: RingBuffer<1> = RingBuffer::new();

        assert!(!ring_buffer.is_full());
    }

    #[test]
    fn test_len_correctly_returns_length_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<1> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        ring_buffer.push_back(5).unwrap();

        assert_eq!(ring_buffer.len(), 1);
    }

    #[test]
    fn test_push_back_correctly_appends_element_to_back_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_back(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);
        
        let push_back_result = ring_buffer.push_back(8);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);
        assert_eq!(ring_buffer[1], 8);
    }

    #[test]
    fn test_push_back_errors_full_ring_buffer() {
        let mut ring_buffer: RingBuffer<1> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_back(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);

        let push_back_result = ring_buffer.push_back(12);

        assert!(push_back_result.is_err());
        assert_eq!(push_back_result.err().unwrap(), OrderBookError::FullRingBuffer);
    }

    #[test]
    fn test_push_front_correctly_apppends_element_to_front_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_front_result = ring_buffer.push_front(5);

        for i in ring_buffer.buf {
            println!("{i}");
        }

        assert!(push_front_result.is_ok());
        assert_eq!(ring_buffer[127], 5);
        
        let push_front_result = ring_buffer.push_front(8);

        assert!(push_front_result.is_ok());
        assert_eq!(ring_buffer[126], 8);
        assert_eq!(ring_buffer[127], 5);
    }

    #[test]
    fn test_push_front_errors_full_ring_buffer() {
        let mut ring_buffer: RingBuffer<1> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_front_result = ring_buffer.push_front(5);

        assert!(push_front_result.is_ok());
        assert_eq!(ring_buffer[0], 5);

        let push_front_result = ring_buffer.push_front(12);

        assert!(push_front_result.is_err());
        assert_eq!(push_front_result.err().unwrap(), OrderBookError::FullRingBuffer);
    }

    #[test]
    fn test_pop_back_correctly_removes_element_from_back_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_front(5);

        println!("{}", ring_buffer.len());

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[127], 5);
        
        let push_back_result = ring_buffer.push_front(8);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[127], 5);
        assert_eq!(ring_buffer[126], 8);

        let pop_back_result = ring_buffer.pop_back();

        assert!(pop_back_result.is_ok());
        assert_eq!(ring_buffer.len(), 1);
        assert_eq!(ring_buffer[126], 8);
    }

    #[test]
    fn test_pop_back_errors_empty_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let pop_back_result = ring_buffer.pop_back();

        assert!(pop_back_result.is_err());
        assert_eq!(pop_back_result.err().unwrap(), OrderBookError::EmptyRingBuffer);
        assert_eq!(ring_buffer.len(), 0);
    }

    #[test]
    fn test_pop_front_correctly_removes_element_from_front_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_front(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer.len(), 1);
        assert_eq!(ring_buffer[127], 5);
        
        let push_back_result = ring_buffer.push_front(8);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer.len(), 2);
        assert_eq!(ring_buffer[126], 8);
        assert_eq!(ring_buffer[127], 5);

        let pop_front_result = ring_buffer.pop_front();

        assert!(pop_front_result.is_ok());
        assert_eq!(ring_buffer.len(), 1);
        assert_eq!(ring_buffer[127], 5);
    }

    #[test]
    fn test_pop_front_errors_empty_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let pop_front_result = ring_buffer.pop_front();

        assert!(pop_front_result.is_err());
        assert_eq!(pop_front_result.err().unwrap(), OrderBookError::EmptyRingBuffer);
    }

    #[test]
    fn test_front_correctly_returns_front_element_for_ring_buffer_with_elements() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_back(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);

        let front_result = ring_buffer.front();

        assert!(front_result.is_some());
        assert_eq!(front_result.unwrap(), 5);
    }

    #[test]
    fn test_front_correctly_returns_none_for_empty_ring_buffer() {
        let ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let front_result = ring_buffer.front();

        assert!(front_result.is_none());
    }

    #[test]
    fn test_remove_by_value_correctly_removes_element_with_specified_value() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_back(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);

        let push_back_result = ring_buffer.push_back(8);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[1], 8);

        let push_back_result = ring_buffer.push_back(12);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[2], 12);

        let remove_by_value_result = ring_buffer.remove_by_value(8);

        assert!(remove_by_value_result);
        assert_eq!(ring_buffer.len(), 2);
        assert_eq!(ring_buffer[0], 5);
        assert_eq!(ring_buffer[1], 12);
    }

    #[test]
    fn test_iter_correctly_returns_iterator_of_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let push_back_result = ring_buffer.push_back(5);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[0], 5);

        let push_back_result = ring_buffer.push_back(8);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[1], 8);

        let push_back_result = ring_buffer.push_back(12);

        assert!(push_back_result.is_ok());
        assert_eq!(ring_buffer[2], 12);

        let mut iter_result = ring_buffer.iter();

        assert_eq!(iter_result.next().unwrap(), 5);
        assert_eq!(iter_result.next().unwrap(), 8);
        assert_eq!(iter_result.next().unwrap(), 12);
    }

    #[test]
    fn test_remove_by_value_correctly_returns_false_with_value_not_in_ring_buffer() {
        let mut ring_buffer: RingBuffer<128> = RingBuffer::new();

        assert_eq!(ring_buffer.len(), 0);

        let remove_by_value_result = ring_buffer.remove_by_value(5);

        assert!(!remove_by_value_result);
    }
}