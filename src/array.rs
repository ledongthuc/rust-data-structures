use crate::errors::RunOutOfCapacity;
use core::ptr;
use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, ops::Index};
use core::marker::PhantomData;

pub struct StaticHeapArray<T> {
    mem_layout: Layout,
    pointer: *mut T,
    cap: usize,
    size: usize,
}

impl<T> StaticHeapArray<T> {
    pub fn new(cap: usize) -> StaticHeapArray<T> {
        let mem_layout = Layout::array::<T>(cap).unwrap();
        let ptr: *mut u8 = unsafe { alloc(mem_layout) };
        if ptr.is_null() {
            handle_alloc_error(mem_layout);
        }
        StaticHeapArray {
            mem_layout,
            pointer: ptr as *mut T,
            cap,
            size: 0,
        }
    }

    pub fn from<const SIZE: usize>(arr: [T; SIZE]) -> StaticHeapArray<T> {
        let mut r = StaticHeapArray::new(SIZE);
        for item in arr {
            r.push(item).unwrap();
        }
        r
    }

    pub fn push(&mut self, item: T) -> Result<(), RunOutOfCapacity> {
        if self.is_full() {
            return Err(RunOutOfCapacity {});
        }
        unsafe { ptr::write(self.pointer.add(self.size), item) }
        self.size += 1;
        Ok(())
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.get_size() == self.get_cap()
    }

    pub fn get_ref(&self, index: usize) -> Option<&T> {
        match self.is_out_of_index(index) {
            true => None,
            false => Some(unsafe { &*self.pointer.add(index) as &T }),
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.is_out_of_index(index) {
            true => None,
            false => Some(unsafe { &mut *self.pointer.add(index) as &mut T }),
        }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        match self.is_out_of_index(index) {
            true => None,
            false => Some(unsafe { ptr::read(self.pointer.add(index)) }),
        }
    }

    #[inline]
    pub fn is_out_of_index(&self, index: usize) -> bool {
        index >= self.get_size()
    }

    #[inline]
    pub fn get_cap(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn iter(&self) -> StaticHeapArrayIter<'_, T> {
        StaticHeapArrayIter::new(self)
    }
}

impl<T> Drop for StaticHeapArray<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer as *mut u8, self.mem_layout) }
    }
}

impl<T> Index<usize> for StaticHeapArray<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        self.get_ref(idx).unwrap()
    }
}

pub struct StaticHeapArrayIter<'a, T> {
    s: &'a StaticHeapArray<T>,
    reading_index: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> StaticHeapArrayIter<'a, T> {
    pub fn new(array: &'a StaticHeapArray<T>) -> Self {
        Self {
            s: array,
            reading_index: 0,
            _marker: PhantomData,
        }
    }
}
impl<'a, T> Iterator for StaticHeapArrayIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_out_of_index(self.reading_index) {
            return None;
        }
        let result = self.s.get(self.reading_index);
        self.reading_index += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::array::*;
    use crate::errors::*;

    #[test]
    fn test_static_heap_array_new() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::new(5);

        arr.push(1).unwrap();
        arr.push(2).unwrap();
        arr.push(3).unwrap();
        arr.push(4).unwrap();
        arr.push(5).unwrap();
        assert!(arr.is_full());
        assert_eq!(RunOutOfCapacity {}, arr.push(6).unwrap_err());

        assert_eq!(1, arr.get(0).unwrap());
        assert_eq!(2, arr.get(1).unwrap());
        assert_eq!(3, arr.get(2).unwrap());
        assert_eq!(4, arr.get(3).unwrap());
        assert_eq!(5, arr.get(4).unwrap());
        assert!(arr.is_out_of_index(6));
        assert_eq!(None, arr.get(6));

        assert_eq!(&1, arr.get_ref(0).unwrap());
        assert_eq!(&2, arr.get_ref(1).unwrap());
        assert_eq!(&3, arr.get_ref(2).unwrap());
        assert_eq!(&4, arr.get_ref(3).unwrap());
        assert_eq!(&5, arr.get_ref(4).unwrap());
        assert_eq!(None, arr.get(6));
    }

    #[test]
    fn test_static_heap_array_from_initialed_array() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::from([1, 2, 3, 4, 5]);

        assert!(arr.is_full());
        assert_eq!(RunOutOfCapacity {}, arr.push(6).unwrap_err());

        assert_eq!(1, arr.get(0).unwrap());
        assert_eq!(2, arr.get(1).unwrap());
        assert_eq!(3, arr.get(2).unwrap());
        assert_eq!(4, arr.get(3).unwrap());
        assert_eq!(5, arr.get(4).unwrap());
        assert!(arr.is_out_of_index(6));
        assert_eq!(None, arr.get(6));
    }

    #[test]
    fn test_static_heap_array_get_mut() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::from([1, 2, 3, 4, 5]);

        assert_eq!(3, arr.get(2).unwrap());

        let item3 = arr.get_mut(2).unwrap();
        *item3 = 99;
        assert_eq!(99, arr.get(2).unwrap());
    }

    #[test]
    fn test_static_heap_array_index_access() {
        let arr: StaticHeapArray<i32> = StaticHeapArray::from([1, 2, 3, 4, 5]);

        assert_eq!(1, arr[0]);
        assert_eq!(2, arr[1]);
        assert_eq!(3, arr[2]);
        assert_eq!(4, arr[3]);
        assert_eq!(5, arr[4]);
    }

    #[test]
    fn test_static_heap_array_iter() {
        let arr: StaticHeapArray<i32> = StaticHeapArray::from([1, 2, 3, 4, 5]);

        let mut iter = arr.iter();
        for i in 0..5 {
            assert_eq!(i + 1, iter.next().unwrap());
        }
        assert!(iter.next().is_none());

        let mut i: i32 = 0;
        for item in arr.iter() {
            assert_eq!(i + 1, item);
            i += 1;
        }
        assert!(iter.next().is_none());
    }
}
