pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

// Impl sync for SpinLock where T is send
// T impls send -> Can be safely sent to different threads
// T being send is sufficient as T will only ever be mutably owned by a singular thread at a time
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
        // NOTE: Using a compare and exchange (CAS) here might be easier to reason about
        while self.lock.swap(true, Ordering::Acquire) {
            // Informing the CPU that we're in a spin loop, allowing it to optimize accordingly
            std::hint::spin_loop()
        }

        Guard { lock: self }
    }

    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release)
    }
}

// Implementation of guard, that derefs to type T for easier usage
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

// Unlocking the spinlock when the guard is dropped
impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release)
    }
}
