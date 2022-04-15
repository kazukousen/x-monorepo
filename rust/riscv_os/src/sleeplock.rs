use core::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
};

use crate::{cpu::CPU_TABLE, process::PROCESS_TABLE, spinlock::SpinLock};

pub struct SleepLock<T> {
    inner_lock: SpinLock<()>,
    locked: Cell<bool>,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SleepLock<T> {}

impl<T> SleepLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner_lock: SpinLock::new(()),
            locked: Cell::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SleepLockGuard<'_, T> {
        let mut guard = self.inner_lock.lock();

        while self.locked.get() {
            unsafe {
                guard = CPU_TABLE
                    .my_proc()
                    .sleep(self.locked.as_ptr() as usize, guard);
            };
        }

        self.locked.set(true);
        drop(guard);

        SleepLockGuard {
            lock: &self,
            data: unsafe { &mut (*self.data.get()) },
        }
    }

    /// called by its guard when dropped
    fn unlock(&self) {
        let guard = self.inner_lock.lock();
        self.locked.set(false);
        unsafe { PROCESS_TABLE.wakeup(self.locked.as_ptr() as usize) };
        drop(guard);
    }
}

pub struct SleepLockGuard<'a, T> {
    lock: &'a SleepLock<T>,
    data: &'a mut T,
}

impl<'a, T> Deref for SleepLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl<'a, T> DerefMut for SleepLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T> Drop for SleepLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
