use crate::errors::{OutOfIndex, RunOutOfCapacity};
use core::ptr;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

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

    pub fn is_full(&self) -> bool {
        self.size == self.cap
    }

    pub fn index_of(&self, index: usize) -> Result<T, OutOfIndex> {
        if self.is_out_of_index(index) {
            return Err(OutOfIndex {});
        }
        unsafe { Ok(ptr::read(self.pointer.add(index))) }
    }

    pub fn is_out_of_index(&self, index: usize) -> bool {
        index >= self.size
    }
}

impl<T> Drop for StaticHeapArray<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer as *mut u8, self.mem_layout) }
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
}
