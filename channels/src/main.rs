use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        Arc,
    },
};

fn main() {
    println!("Hello, world!");
}

// One shot channels implementation
// Requirements:
//     - Sender can send a message to a receiver listening on the same channel
//     - Sender can only send a message once - Type system should enforce this
//     - Unreceived messages should be dropped if the channel is dropped?

// Interface is as follows:
// - Channel::new() -> (Sender, Receiver)
// - Sender::send(T) -> Change shared message state
// - Receiver::receive(T) -> Spin lock on state, if not valid state, reattempt with some timeout
//                        -> Read shared memory location if state valid

// Possible message states
const EMPTY: u8 = 0;
const READY: u8 = 1;
const READING: u8 = 2;
const READ: u8 = 3;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        state: AtomicU8::new(0),
        data: UnsafeCell::new(MaybeUninit::uninit()),
    });

    (
        Sender {
            channel: channel.clone(),
        },
        Receiver { channel },
    )
}

struct Channel<T> {
    // Possible message states.
    // Update value so receiver may know when it's able to receive message
    state: AtomicU8,
    // Should be a pointer to some shared data (message)
    data: UnsafeCell<MaybeUninit<T>>,
}

// NOTE: DO NOT UNDERSTAND FULLY
unsafe impl<T> Sync for Channel<T> where T: Send {}

struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    // Consume self, so method can only be called once.
    pub fn send(self, message: T) {
        // Write message to shared memory location
        // Do a release store to update the message status indicating its ready
        unsafe { *self.channel.data.get() }.write(message);

        // Should be a CAS to ensure no other state can happen?
        self.channel.state.store(READY, Ordering::Release);
    }
}

struct Receiver<T> {
    channel: Arc<Channel<T>>,
}
impl<T> Receiver<T> {
    // CAS operation to check if state of message is ready for consumption
    // If ready for consumption -> return the message
    pub fn receive(&self) -> &T {
        while self
            .channel
            .state
            .compare_exchange(READY, READING, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }

        unsafe { &(*self.channel.data.get()).assume_init() }
    }
}
