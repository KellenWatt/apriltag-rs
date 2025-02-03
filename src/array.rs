use crate::native::*;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

extern "C" {
    fn zarray_create(el_sz: ::std::os::raw::c_int) -> *mut zarray_t;
    fn zarray_destroy(za: *mut zarray_t);
    fn zarray_copy(za: *const zarray_t) -> *mut zarray_t;
    fn zarray_size(za: *const zarray_t) -> ::std::os::raw::c_int;
    fn zarray_ensure_capacity(za: *mut zarray_t, capacity: ::std::os::raw::c_int);
    fn zarray_add(za: *mut zarray_t, p: *const ::std::os::raw::c_void);
    fn zarray_get_volatile(za: *const zarray_t, idx: ::std::os::raw::c_int, p: *mut ::std::os::raw::c_void);
    fn zarray_get(za: *const zarray_t, idx: ::std::os::raw::c_int, p: *mut ::std::os::raw::c_void);
    fn zarray_truncate(za: *mut zarray_t, sz: ::std::os::raw::c_int);
    fn zarray_remove_index(za: *mut zarray_t, idx: ::std::os::raw::c_int, shuffle: ::std::os::raw::c_int);
    fn zarray_insert(za: *mut zarray_t, idx: ::std::os::raw::c_int, p: *const ::std::os::raw::c_void);
    fn zarray_set(za: *mut zarray_t, idx: ::std::os::raw::c_int, p: *const ::std::os::raw::c_void);
    fn zarray_clear(za: *mut zarray_t);
    fn zarray_index_of(za: *const zarray_t, p: *const ::std::os::raw::c_void) -> ::std::os::raw::c_int;
}

macro_rules! void_ptr {
    ($p:expr) => {
        $p as *const ::std::os::raw::c_void
    }
}

macro_rules! void_mut {
    ($p:expr) => {
        $p as *mut ::std::os::raw::c_void
    }
}

#[allow(dead_code)]
pub struct Array<T> {
    raw: *mut zarray_t,
    _type: PhantomData<T>,
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {zarray_destroy(self.raw)};
    }
}

impl<T> Clone for Array<T> {
    fn clone(&self) -> Array<T> {
        Array {
            raw: unsafe{zarray_copy(self.raw)},
            _type: self._type,
        }
    }
}

#[allow(dead_code)]
impl<T> Array<T> {

    pub unsafe fn from_raw(ptr: *mut zarray_t) -> Array<T> {
        Array {
            raw: ptr,
            _type: PhantomData,
        }
    }

    pub fn new() -> Option<Array<T>> {
        let raw = unsafe{zarray_create(std::mem::size_of::<T>() as i32)};
        if raw.is_null() {
            return None;
        }
        Some(Array {
            raw,
            _type: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        unsafe {
            zarray_size(self.raw) as usize
        }
    }

    pub fn reserve(&mut self, capacity: i32) {
        unsafe {
            zarray_ensure_capacity(self.raw, capacity);
        }
    }

    pub fn truncate(&mut self, size: i32) {
        unsafe {
            zarray_truncate(self.raw, size);
        }
    }

    pub fn push(&mut self, value: T) {
        unsafe {
            let p = void_ptr!((&value) as *const T);
            zarray_add(self.raw, p);
        }
    }

    pub unsafe fn get_value_unchecked(&self, index: usize) -> T {
        let mut out = MaybeUninit::<T>::uninit();
        zarray_get(self.raw, index as i32, void_mut!(out.as_mut_ptr()));
        out.assume_init()
    }

    pub fn get_value(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        Some(unsafe{self.get_value_unchecked(index)})
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let mut out = MaybeUninit::<T>::uninit();
        zarray_get_volatile(self.raw, index as i32, void_mut!(out.as_mut_ptr()));
        out.as_ptr().as_ref().unwrap_unchecked()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        Some(unsafe{self.get_unchecked(index)})
    }
   

    pub unsafe fn get_mut_unchecked(&self, index: usize) -> &mut T {
        let mut out = MaybeUninit::<T>::uninit();
        zarray_get_volatile(self.raw, index as i32, out.as_mut_ptr() as *mut ::std::os::raw::c_void);
        out.as_mut_ptr().as_mut().unwrap_unchecked()
    }

    pub fn get_mut(&self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        Some(unsafe{self.get_mut_unchecked(index)})
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len() {
            panic!("index out of bounds");
        }

        unsafe {
            let out = self.get_value_unchecked(index);
            zarray_remove_index(self.raw, index as i32, 0);
            out
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len() {
            panic!("index out of bounds");
        }

        unsafe {
            let out = self.get_value_unchecked(index);
            zarray_remove_index(self.raw, index as i32, 1);
            out
        }
    }

    // TODO remove_value (zarray_remove_value)
    // pending: find/index_of


    pub fn insert(&mut self, index: usize, value: T) {
        if index >= self.len() {
            panic!("index out of bounds");
        }

        unsafe {
            let p = (&value) as *const T;
            zarray_insert(self.raw, index as i32, void_ptr!(p));
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.len() {
            panic!("index out of bounds");
        }

        unsafe {
            let p = (&value) as *const T;
            zarray_set(self.raw, index as i32, void_ptr!(p));
        }
    }

    pub fn clear(&mut self) {
        unsafe{zarray_clear(self.raw)};
    }

    pub fn find(&mut self, value: &T) -> Option<usize> {
        unsafe {
            let p = value as *const T;
            let index = zarray_index_of(self.raw, void_ptr!(p));
            if index < 0 {
                None
            } else {
                Some(index as usize)
            }
        }
    }

    pub fn contains(&mut self, value: &T) -> bool {
        self.find(value).is_some()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            data: self,
            current: 0,
        }
    }

}

pub struct Iter<'a, T> {
    data: &'a Array<T>,
    current: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.data.get(self.current);
        self.current += 1;
        out
    }
}
