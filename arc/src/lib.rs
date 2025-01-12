use std::{
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                count: AtomicU32::new(0),
                data: value,
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    // Function invoked as Arc::get_mut() instead of a.get_mut()
    // It's advised to implement functions like so for types that implement Deref to avoid ambiguity with a similarly
    // defined method on the underlying T
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().count.load(Ordering::Acquire) == 1 {
            unsafe { Some(&mut arc.ptr.as_mut().data) }
        } else {
            None
        }
    }
}

pub struct ArcData<T> {
    count: AtomicU32,
    data: T,
}

unsafe impl<T: Sync + Send> Sync for Arc<T> {}
unsafe impl<T: Sync + Send> Send for Arc<T> {}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.data().count.fetch_add(1, Ordering::Acquire);

        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let v = self.data().count.fetch_sub(1, Ordering::Acquire);
        if v == 1 {
            unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data().data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
