use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_all, wake_one};

pub struct RwLock<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
    writer_beacon: AtomicU32,
}

impl<T> RwLock<T> {
    fn read(&self) -> ReadGuard<T> {
        // NOTE: Concerned that state may change between the load + processing operations.
        // Not entirely atomic

        // CAS operation after the state.load() addresses the above concerns
        let mut s = self.state.load(Ordering::Acquire);

        loop {
            if s < u32::MAX {
                assert_ne!(s, u32::MAX - 1, "Too many readers");

                match self
                    .state
                    .compare_exchange(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
                {
                    Ok(_) => return ReadGuard { lock: self },
                    Err(e) => s = e,
                }

                if s == u32::MAX {
                    wait(&self.state, u32::MAX);
                    s = self.state.load(Ordering::Acquire);
                }
            }
        }
    }

    fn write(&mut self) -> WriteGuard<T> {
        while let Err(e) =
            self.state
                .compare_exchange(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed)
        {
            let writer_beacon = self.state.load(Ordering::Acquire);
            if self.state.load(Ordering::Acquire) != 0 {
                wait(&self.writer_beacon, writer_beacon)
            }
        }

        WriteGuard { lock: self }
    }
}

struct ReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.lock.state.fetch_sub(1, Ordering::Acquire) == 1 {
            // Wake a potential waiting writer
            self.lock.writer_beacon.fetch_add(1, Ordering::Release);
            wake_one(&self.lock.writer_beacon);
        }
    }
}

struct WriteGuard<'a, T> {
    lock: &'a mut RwLock<T>,
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        // Free the lock
        self.lock.state.store(0, Ordering::Release);
        // First wake a potential waiting writer
        self.lock.writer_beacon.fetch_add(1, Ordering::Release);
        wake_one(&self.lock.writer_beacon);
        // Then wake all waiting readers
        wake_all(&self.lock.state);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rwtest() {}
}
