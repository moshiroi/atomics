use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicU32,
};

pub struct RwLock<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
    waiting_writers: AtomicU32,
}

impl<T> RwLock<T> {
    fn read(&self) -> ReadGuard<T> {
        // If state is even, state += 2
        // If state is odd, you must not dish out any further readers. wait.
        todo!()
    }

    fn write(&mut self) -> WriteGuard<T> {
        // If state is odd, there is already a writer waiting - sleep
        // If state is even, state += 1, wait
        // If state == 1, return WriteGuard
        todo!()
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
    fn drop(&mut self) {}
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
