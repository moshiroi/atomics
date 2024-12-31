use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

mod lib;

use lib::SpinLock;

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
