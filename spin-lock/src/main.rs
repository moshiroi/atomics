use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

fn main() {
    let spin_lock: &'static _ = Box::leak(Box::new(SpinLock::new(0)));

    let mut threads = vec![];
    for _ in 0..10 {
        let t = thread::spawn(|| {
            let mut g = spin_lock.lock();
            for _ in 0..25 {
                *g += 1;
            }
        });

        threads.push(t);
    }

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(*spin_lock.lock(), 250);
}

pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

// Impl sync for SpinLock where T is send
// T impls send -> Can be safely sent to different threads
// Implementing Sync -> Can safely be shared among threads
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        while self.lock.swap(true, Ordering::Acquire) {
            std::hint::spin_loop()
        }

        Guard { lock: self }
    }

    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release)
    }
}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T>
where
    T: Send,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Guard<'_, T>
where
    T: Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release)
    }
}
