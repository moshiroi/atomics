use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use atomic_wait::{wait, wake_all};

struct Channel<T> {
    state: AtomicU32,
    message: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for Channel<T> {}
pub fn channel<T>() -> (Reader<T>, Writer<T>) {
    let channel = Arc::new(Channel {
        state: AtomicU32::new(0),
        message: UnsafeCell::new(MaybeUninit::uninit()),
    });

    (
        Reader {
            channel: Arc::clone(&channel),
        },
        Writer {
            channel: Arc::clone(&channel),
        },
    )
}

struct Reader<T> {
    channel: Arc<Channel<T>>,
}

unsafe impl<T: Send> Send for Reader<T> {}

impl<T: Send> Reader<T> {
    fn read(&self) -> T {
        // Check if state == 1 -> Ready for reading
        while self.channel.state.load(Ordering::Acquire) != 1 {
            // Wait until message has been sent i.e state -> 1
            wait(&self.channel.state, 0)
        }

        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Reader<T> {
    fn drop(&mut self) {}
}

struct Writer<T> {
    channel: Arc<Channel<T>>,
}
unsafe impl<T: Send> Send for Writer<T> {}
impl<T: Send> Writer<T> {
    fn send(self, message: T) {
        if let Err(e) =
            self.channel
                .state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            panic!("Invalid state, cannot send message: state is {e}")
        }

        unsafe { (*self.channel.message.get()).write(message) };
        // Wake potential waiting reader(s)
        wake_all(&self.channel.state)
    }
}
impl<T> Drop for Writer<T> {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::channel;
    use std::{thread, time::Duration};

    #[test]
    fn read_write() {
        let (reader, writer) = channel::<String>();

        let reader_thread = thread::spawn(move || {
            println!("Reader waiting to receive message");
            let message = reader.read();
            assert_eq!(message, "It's working".to_owned());
            println!("message is: {message}");
        });

        thread::sleep(Duration::from_millis(500));

        let writer_thread = thread::spawn(move || {
            writer.send("It's working".to_string());
        });

        reader_thread.join().unwrap();
        writer_thread.join().unwrap();
    }
}
