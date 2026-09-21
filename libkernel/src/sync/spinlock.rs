use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// A spinlock provides interior mutability to a value,
/// guaranteeing exclusive ownership for the duration that the lock is acquired
/// by making other threads spin and wait their turn.
pub struct SpinLock<T> {
    value: UnsafeCell<T>,
    available: AtomicBool,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            available: AtomicBool::new(true),
        }
    }

    /// Attempt to acquire the lock, if it is already in use by another thread
    /// this might cause you to spin and wait until it becomes available.
    ///
    /// It is very possible to end up in a deadlock where two threads are waiting for
    /// eachother's resources to become available.
    pub fn lock<'a>(&'a self) -> LockGuard<'a, T> {
        while self
            .available
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        LockGuard { source: self }
    }
}

unsafe impl<T: Send> Sync for SpinLock<T> {}

/// The LockGuard is a proof of exclusive ownership acquired by calling [`SpinLock::lock()`]
/// When it is dropped, the lock is automatically released.
pub struct LockGuard<'a, T> {
    source: &'a SpinLock<T>,
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.source.available.store(true, Ordering::Release);
    }
}

impl<'a, T> Deref for LockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.source.value.get() }
    }
}

impl<'a, T> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.source.value.get() }
    }
}
