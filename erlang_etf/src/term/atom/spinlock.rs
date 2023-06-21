use std::hint::spin_loop;
use std::sync::atomic::{AtomicUsize, Ordering};

const WRITER_BIT: usize = isize::min_value() as usize;

pub(crate) struct SpinRwLock {
    lock: AtomicUsize,
}
impl SpinRwLock {
    pub(crate) fn new() -> Self {
        Self {
            lock: AtomicUsize::new(0),
        }
    }

    pub(crate) fn read(&self) -> SpinRwLockReadGuard {
        while {
            let mut lock;

            // Wait for writer to drop before doing CAS
            while {
                lock = self.lock.load(Ordering::Relaxed);
                lock & WRITER_BIT != 0
            } {
                spin_loop();
            }

            // Unset writer
            lock &= !WRITER_BIT;

            // Increment reader count
            if let Ok(_) = self.lock.compare_exchange_weak(lock, lock + 1, Ordering::SeqCst, Ordering::SeqCst) {
                false
            } else {
                true
            }
        } {
            spin_loop();
        }

        SpinRwLockReadGuard { lock: &self.lock }
    }

    pub(crate) fn write(&self) -> SpinRwLockWriteGuard {
        loop {
            // Attempt to set the writer bit. CAS ensures that the writer bit will not be
            // successfully set unless it is currently not set.
            let old = (!WRITER_BIT) & self.lock.load(Ordering::Relaxed);
            let new = WRITER_BIT | old;

            if let Ok(_) = self.lock.compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::SeqCst) {
                // Wait for all readers to drop
                while self.lock.load(Ordering::Relaxed) != WRITER_BIT {
                    spin_loop();
                }
                break;
            }
        }

        SpinRwLockWriteGuard { lock: &self.lock }
    }
}

unsafe impl Send for SpinRwLock {}
unsafe impl Sync for SpinRwLock {}

pub(crate) struct SpinRwLockReadGuard<'a> {
    lock: &'a AtomicUsize,
}

impl<'a> Drop for SpinRwLockReadGuard<'a> {
    fn drop(&mut self) {
        self.lock.fetch_sub(1, Ordering::SeqCst);
    }
}

pub(crate) struct SpinRwLockWriteGuard<'a> {
    lock: &'a AtomicUsize,
}

impl<'a> Drop for SpinRwLockWriteGuard<'a> {
    fn drop(&mut self) {
        self.lock.store(0, Ordering::Relaxed);
    }
}
