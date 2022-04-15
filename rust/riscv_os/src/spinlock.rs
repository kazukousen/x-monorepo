use core::{
    cell::UnsafeCell,
    ops::Deref,
    ops::DerefMut,
    sync::atomic::{fence, AtomicBool, Ordering},
};

use crate::cpu;

pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SpinLock<T> {
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        self.acquire();
        SpinLockGuard {
            inner: &self,
            data: unsafe { &mut *self.data.get() },
        }
    }

    // TODO: fn holding()

    fn acquire(&self) {
        cpu::push_off();

        // TODO: if !self.holding() panic
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {}
        fence(Ordering::SeqCst);
    }

    fn release(&self) {
        // TODO: if !self.holding() panic
        fence(Ordering::SeqCst);
        self.lock.store(false, Ordering::Release);

        cpu::pop_off();
    }

    pub fn unlock(&self) {
        self.release();
    }
}

pub struct SpinLockGuard<'a, T: ?Sized> {
    inner: &'a SpinLock<T>,
    data: &'a mut T,
}

impl<'a, T: ?Sized> SpinLockGuard<'a, T> {
    pub fn weak(self) -> SpinLockWeakGuard<'a, T> {
        SpinLockWeakGuard{
            inner: self.inner,
        }
    }
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.inner.release();
    }
}

pub struct SpinLockWeakGuard<'a, T: ?Sized> {
    inner: &'a SpinLock<T>,
}

impl<'a, T: ?Sized> SpinLockWeakGuard<'a, T> {
    pub fn lock(self) -> SpinLockGuard<'a, T> {
        self.inner.lock()
    }
}

