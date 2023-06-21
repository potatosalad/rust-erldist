use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

pub trait FromRawPtr {
    unsafe fn from_raw_ptr(ptr: *mut ()) -> Self;
}

pub trait IntoRawPtr {
    fn into_raw_ptr(self) -> *mut ();
}

impl<T> IntoRawPtr for Arc<T> {
    #[inline]
    fn into_raw_ptr(self) -> *mut () {
        Arc::into_raw(self) as *mut T as *mut ()
    }
}

impl<T> FromRawPtr for Arc<T> {
    #[inline]
    unsafe fn from_raw_ptr(ptr: *mut ()) -> Arc<T> {
        Arc::from_raw(ptr as *const () as *const T)
    }
}

pub trait AtomicOptionRefTrait: FromRawPtr + IntoRawPtr + Clone {}

#[derive(Debug)]
pub struct AtomicOptionRef<T>(AtomicPtr<T>)
where
    T: AtomicOptionRefTrait;
impl<T> AtomicOptionRef<T>
where
    T: AtomicOptionRefTrait,
{
    pub const fn empty() -> Self {
        Self(AtomicPtr::new(std::ptr::null_mut()))
    }

    pub fn new(value: Option<T>) -> Self {
        Self(AtomicPtr::new(Self::inner_into_raw_ptr(value)))
    }

    pub fn is_some(&self) -> bool {
        !self.0.load(Ordering::SeqCst).is_null()
    }

    pub fn load(&self) -> Option<T> {
        Self::inner_from_raw_ptr(self.0.load(Ordering::SeqCst), true)
    }

    /// Stores the value.
    pub fn store(&self, value: Option<T>) {
        self.swap(value);
    }

    pub fn store_if_none(&self, value: T) -> bool {
        let new = Self::inner_into_raw_ptr(Some(value));
        match self.0.compare_exchange(
            std::ptr::null_mut(),
            new,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Swaps the value, returning the previous value.
    pub fn swap(&self, value: Option<T>) -> Option<T> {
        Self::inner_from_raw_ptr(
            self.0
                .swap(Self::inner_into_raw_ptr(value), Ordering::SeqCst),
            false,
        )
    }

    #[inline]
    fn inner_into_raw_ptr(value: Option<T>) -> *mut T {
        match value {
            Some(value) => T::into_raw_ptr(value) as *mut T,
            None => std::ptr::null_mut(),
        }
    }

    #[inline]
    fn inner_from_raw_ptr(ptr: *mut T, increment: bool) -> Option<T> {
        if ptr.is_null() {
            None
        } else {
            let value = unsafe { T::from_raw_ptr(ptr as *mut ()) };
            if increment {
                std::mem::forget(T::clone(&value));
            }
            Some(value)
        }
    }
}
impl<T> Clone for AtomicOptionRef<T>
where
    T: AtomicOptionRefTrait,
{
    fn clone(&self) -> Self {
        Self::new(self.load())
    }
}
impl<T> Default for AtomicOptionRef<T>
where
    T: AtomicOptionRefTrait,
{
    fn default() -> Self {
        Self::empty()
    }
}
impl<T> Drop for AtomicOptionRef<T>
where
    T: AtomicOptionRefTrait,
{
    fn drop(&mut self) {
        let ptr = self.0.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                // Reconstruct the Arc from the raw ptr which will trigger our destructor
                // if there is one
                let _ = T::from_raw_ptr(ptr as *mut ());
            }
        }
    }
}

// use std::sync::atomic::{AtomicPtr, Ordering};

// use self::spinlock::*;

// use super::Atom;

// #[derive(Debug)]
// pub struct AtomicAtomPtr {
//     ptr: AtomicPtr<Atom>,
//     lock: SpinRwLock,
// }
// impl AtomicAtomPtr {
//     /// Creates a new atomic reference with `None` initial value.
//     pub fn new() -> Self {
//         Self::default()
//     }

//     /// Creates a new atomic reference from the given initial value.
//     pub fn from(value: Option<Atom>) -> Self {
//         Self {
//             ptr: AtomicPtr::new(option_atom_to_ptr(value)),
//             lock: SpinRwLock::new(),
//         }
//     }

//     /// Returns `true` if the optional reference has `Some` value.
//     pub fn is_some(&self) -> bool {
//         self.ptr.load(Ordering::SeqCst).is_null()
//     }

//     /// Loads and returns a reference to the value or `None`
//     /// if the value is not set.
//     pub fn load(&self) -> Option<Atom> {
//         let _guard = self.lock.read();
//         ptr_to_option_atom(self.ptr.load(Ordering::SeqCst), true)
//     }

//     /// Stores the value.
//     pub fn store(&self, value: Option<Atom>) {
//         self.swap(value);
//     }

//     /// Swaps the value, returning the previous value.
//     pub fn swap(&self, value: Option<Atom>) -> Option<Atom> {
//         let _guard = self.lock.write();
//         ptr_to_option_atom(
//             self.ptr.swap(option_atom_to_ptr(value), Ordering::SeqCst),
//             false,
//         )
//     }
// }
// impl Default for AtomicAtomPtr {
//     fn default() -> Self {
//         Self::from(None)
//     }
// }
// impl Drop for AtomicAtomPtr {
//     fn drop(&mut self) {
//         let ptr = self.ptr.swap(std::ptr::null_mut(), Ordering::SeqCst);
//         if !ptr.is_null() {
//             unsafe {
//                 // Reconstruct the Arc from the raw ptr which will trigger our destructor
//                 // if there is one
//                 let _ = Atom::from_raw(ptr as *const _);
//             }
//         }
//     }
// }

// fn option_atom_to_ptr(value: Option<Atom>) -> *mut Atom {
//     if let Some(atom) = value {
//         Atom::into_raw(atom) as *mut _
//     } else {
//         std::ptr::null_mut()
//     }
// }

// fn ptr_to_option_atom(ptr: *mut Atom, increment: bool) -> Option<Atom> {
//     if ptr.is_null() {
//         // Return `None` if null is stored in the AtomicPtr
//         None
//     } else {
//         // Otherwise, reconstruct the stored Arc
//         let atom = unsafe { Atom::from_raw(ptr as *const _) };

//         if increment {
//             // Increment the atomic reference count
//             std::mem::forget(Atom::clone(&atom));
//         }

//         // And return our reference
//         Some(atom)
//     }
// }

// mod spinlock {
//     use std::hint::spin_loop;
//     use std::sync::atomic::{AtomicUsize, Ordering};

//     const WRITER_BIT: usize = isize::min_value() as usize;

//     #[derive(Debug)]
//     pub(crate) struct SpinRwLock {
//         lock: AtomicUsize,
//     }
//     impl SpinRwLock {
//         pub(crate) fn new() -> Self {
//             Self {
//                 lock: AtomicUsize::new(0),
//             }
//         }

//         pub(crate) fn read(&self) -> SpinRwLockReadGuard {
//             while {
//                 let mut lock;

//                 // Wait for writer to drop before doing CAS
//                 while {
//                     lock = self.lock.load(Ordering::Relaxed);
//                     lock & WRITER_BIT != 0
//                 } {
//                     spin_loop();
//                 }

//                 // Unset writer
//                 lock &= !WRITER_BIT;

//                 // Increment reader count
//                 if let Ok(_) = self.lock.compare_exchange_weak(lock, lock + 1, Ordering::SeqCst, Ordering::SeqCst) {
//                     false
//                 } else {
//                     true
//                 }
//             } {
//                 spin_loop();
//             }

//             SpinRwLockReadGuard { lock: &self.lock }
//         }

//         pub(crate) fn write(&self) -> SpinRwLockWriteGuard {
//             loop {
//                 // Attempt to set the writer bit. CAS ensures that the writer bit will not be
//                 // successfully set unless it is currently not set.
//                 let old = (!WRITER_BIT) & self.lock.load(Ordering::Relaxed);
//                 let new = WRITER_BIT | old;

//                 if let Ok(_) = self.lock.compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::SeqCst) {
//                     // Wait for all readers to drop
//                     while self.lock.load(Ordering::Relaxed) != WRITER_BIT {
//                         spin_loop();
//                     }
//                     break;
//                 }
//             }

//             SpinRwLockWriteGuard { lock: &self.lock }
//         }
//     }

//     unsafe impl Send for SpinRwLock {}
//     unsafe impl Sync for SpinRwLock {}

//     pub(crate) struct SpinRwLockReadGuard<'a> {
//         lock: &'a AtomicUsize,
//     }

//     impl<'a> Drop for SpinRwLockReadGuard<'a> {
//         fn drop(&mut self) {
//             self.lock.fetch_sub(1, Ordering::SeqCst);
//         }
//     }

//     pub(crate) struct SpinRwLockWriteGuard<'a> {
//         lock: &'a AtomicUsize,
//     }

//     impl<'a> Drop for SpinRwLockWriteGuard<'a> {
//         fn drop(&mut self) {
//             self.lock.store(0, Ordering::Relaxed);
//         }
//     }
// }
