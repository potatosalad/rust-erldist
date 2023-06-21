#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;

pub type ErlDrvSizeT = usize;

#[allow(non_camel_case_types)]
pub type size_t = usize;

extern "C" {
    /// See [driver_alloc](https://www.erlang.org/doc/man/erl_driver.html#driver_alloc) in the Erlang docs.
    pub fn driver_alloc(size: ErlDrvSizeT) -> *mut c_void;
    /// See [driver_free](https://www.erlang.org/doc/man/erl_driver.html#driver_free) in the Erlang docs.
    pub fn driver_free(ptr: *mut c_void);
    /// See [driver_realloc](https://www.erlang.org/doc/man/erl_driver.html#driver_realloc) in the Erlang docs.
    pub fn driver_realloc(ptr: *mut c_void, size: ErlDrvSizeT) -> *mut c_void;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ErlangDriverAlloc;

unsafe impl GlobalAlloc for ErlangDriverAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        driver_alloc(layout.size()) as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        driver_free(ptr as *mut c_void)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        driver_realloc(ptr as *mut c_void, new_size) as *mut u8
    }
}

extern "C" {
    /// See [enif_alloc](http://www.erlang.org/doc/man/erl_nif.html#enif_alloc) in the Erlang docs.
    pub fn enif_alloc(size: size_t) -> *mut c_void;
    /// See [enif_free](http://www.erlang.org/doc/man/erl_nif.html#enif_free) in the Erlang docs.
    pub fn enif_free(ptr: *mut c_void);
    /// See [enif_realloc](http://www.erlang.org/doc/man/erl_nif.html#enif_realloc) in the Erlang docs.
    pub fn enif_realloc(ptr: *mut c_void, size: size_t) -> *mut c_void;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ErlangNifAlloc;

unsafe impl GlobalAlloc for ErlangNifAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        enif_alloc(layout.size()) as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        enif_free(ptr as *mut c_void)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        enif_realloc(ptr as *mut c_void, new_size) as *mut u8
    }
}
