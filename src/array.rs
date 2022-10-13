use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use core::ptr;
use crate::errors::{ OutOfIndex, RunOutOfCapacity };

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

    pub fn from<const SIZE: usize>(arr: [T; SIZE]) -> StaticHeapArray<T>{
        let mut r = StaticHeapArray::new(SIZE);
        for item in arr {
            r.push(item).unwrap();
        }
        r
    }

    pub fn push(&mut self, item: T) -> Result<(), RunOutOfCapacity> {
        if self.size == self.cap {
            return Err(RunOutOfCapacity{});
        }
        unsafe{ ptr::write(self.pointer.add(self.size), item) }
        self.size += 1;
        Ok(())
    }

    pub fn get(&mut self, index: usize) -> Result<T, OutOfIndex> {
        if index >= self.size {
            return Err(OutOfIndex{})
        }
        unsafe{
            Ok(ptr::read(self.pointer.add(index)))
        }
    }
}

impl<T> Drop for StaticHeapArray<T> {
    fn drop(&mut self) {
        unsafe{ dealloc(self.pointer as *mut u8, self.mem_layout) }
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
        assert_eq!(RunOutOfCapacity{}, arr.push(6).unwrap_err());

        assert_eq!(1, arr.get(0).unwrap());
        assert_eq!(2, arr.get(1).unwrap());
        assert_eq!(3, arr.get(2).unwrap());
        assert_eq!(4, arr.get(3).unwrap());
        assert_eq!(5, arr.get(4).unwrap());
        assert_eq!(OutOfIndex{}, arr.get(6).unwrap_err());
    }

    #[test]
    fn test_static_heap_array_from_initialed_array() {
        let mut arr: StaticHeapArray<i32> = StaticHeapArray::from([1,2,3,4,5]);

        assert_eq!(RunOutOfCapacity{}, arr.push(6).unwrap_err());

        assert_eq!(1, arr.get(0).unwrap());
        assert_eq!(2, arr.get(1).unwrap());
        assert_eq!(3, arr.get(2).unwrap());
        assert_eq!(4, arr.get(3).unwrap());
        assert_eq!(5, arr.get(4).unwrap());
        assert_eq!(OutOfIndex{}, arr.get(6).unwrap_err());
    }
}
