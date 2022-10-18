use crate::errors::{OutOfIndex, RunOutOfCapacity};
use core::ptr;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
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
            r.append(item).unwrap();
        }
        r
    }

    pub fn append(&mut self, item: T) -> Result<(), RunOutOfCapacity> {
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

    pub fn index_of(&self, index: usize) -> Result<T, OutOfIndex> {
        if self.is_out_of_index(index) {
            return Err(OutOfIndex {});
        }
        unsafe { Ok(ptr::read(self.pointer.add(index))) }
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
        let result = self.s.index_of(self.reading_index);
        self.reading_index += 1;
        return result.ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::array::*;
    use crate::errors::*;

    #[test]
    fn test_static_heap_array_new() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::new(5);

        arr.append(1).unwrap();
        arr.append(2).unwrap();
        arr.append(3).unwrap();
        arr.append(4).unwrap();
        arr.append(5).unwrap();
        assert!(arr.is_full());
        assert_eq!(RunOutOfCapacity {}, arr.append(6).unwrap_err());

        assert_eq!(1, arr.index_of(0).unwrap());
        assert_eq!(2, arr.index_of(1).unwrap());
        assert_eq!(3, arr.index_of(2).unwrap());
        assert_eq!(4, arr.index_of(3).unwrap());
        assert_eq!(5, arr.index_of(4).unwrap());
        assert!(arr.is_out_of_index(6));
        assert_eq!(OutOfIndex {}, arr.index_of(6).unwrap_err());
    }

    #[test]
    fn test_static_heap_array_from_initialed_array() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::from([1, 2, 3, 4, 5]);

        assert!(arr.is_full());
        assert_eq!(RunOutOfCapacity {}, arr.append(6).unwrap_err());

        assert_eq!(1, arr.index_of(0).unwrap());
        assert_eq!(2, arr.index_of(1).unwrap());
        assert_eq!(3, arr.index_of(2).unwrap());
        assert_eq!(4, arr.index_of(3).unwrap());
        assert_eq!(5, arr.index_of(4).unwrap());
        assert!(arr.is_out_of_index(6));
        assert_eq!(OutOfIndex {}, arr.index_of(6).unwrap_err());
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
