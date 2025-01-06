use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_one};

pub struct Mutex<T> {
    /// 0 - Unlocked
    /// 1 - Locked
    /// 2 - Threads waiting to Lock
    state: AtomicU32,
    value: UnsafeCell<T>,
}

/// Sync for Mutex because we want the mutex to be shared amongst threads,
/// where T: Send because the maximum one thread will have exclusive access to T
unsafe impl<T> Sync for Mutex<T> where T: Send {}

/// Check if its already locked, call wait
/// If unlocked, lock + return guard
/// state 0 -> state 1
/// state 1 -> state 2 + wait
/// state 2 -> state 2 + wait
impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Mutex {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.state.swap(2, Ordering::Acquire) != 0 {
                wait(&self.state, 2)
            }
        }

        MutexGuard { lock: self }
    }

    pub fn unlock(&self) {
        let s = self.state.swap(0, Ordering::AcqRel);
        // If state was = 2, we know other threads are waiting, wake one up
        if s == 2 {
            wake_one(&self.state)
        }
    }
}

/// Mutex::lock -> MutexGuard
pub(crate) struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

/// Deref to &T
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

/// DerefMut to &mut T
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

/// Dropping guard -> unlocks mutex
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::Mutex;

    #[test] // TODO: Fix what appears to be a deadlock
    fn to_1000000() {
        println!("running mutex test");
        let mutex: &'static _ = Box::leak(Box::new(Mutex::new(0)));
        let mut threads = Vec::new();
        for _ in 0..10 {
            let t = thread::spawn(|| {
                for _ in 0..10 {
                    let mut guard = mutex.lock();
                    *guard += 1
                }
            });

            threads.push(t);
        }

        // TODO: Should be a way to join without iterating
        for t in threads {
            t.join().unwrap();
        }

        assert_eq!(100000, *mutex.lock())
    }
}
