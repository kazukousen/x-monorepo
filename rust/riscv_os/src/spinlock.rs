use core::{
    cell::UnsafeCell,
    ops::Deref,
    ops::DerefMut,
    sync::atomic::{fence, AtomicBool, Ordering},
};

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
            lock: &self,
            data: unsafe { &mut *self.data.get() },
        }
    }

    fn acquire(&self) {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {}
        fence(Ordering::SeqCst);
    }

    fn release(&self) {
        fence(Ordering::SeqCst);
        self.lock.store(false, Ordering::Release);
    }
}

pub struct SpinLockGuard<'a, T: ?Sized> {
    lock: &'a SpinLock<T>,
    data: &'a mut T,
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
        self.lock.release();
    }
}
