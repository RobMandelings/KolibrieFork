use std::thread;
use std::time::Duration;

// Newtype wrapper around the raw pointer.
struct SendPtr<T>(*mut T);

impl<T> SendPtr<T> {
    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.0
    }
}

// Assert its okay to send accross threads
unsafe impl<T> Send for SendPtr<T> {}

fn main() {
    // Owner allocates the data on the heap.
    let mut boxed = Box::new(0usize);

    let ptr = SendPtr(&mut *boxed as *mut usize);

    // Spawn a worker thread and move the raw pointer into it.
    let handle = thread::spawn(move || {
        let ptr = ptr; // ptr is SendPtr<usize>, which *is* Send now

        unsafe {
            for _ in 0..10 {
                *ptr.0 += 1; // dereference inner *mut usize
                println!("worker: value = {}", *ptr.0);
            }
        }
        // After this point, the worker no longer uses `ptr`.
    });

    // Owner can do other work in parallel here.
    println!("owner: doing other work...");
    thread::sleep(Duration::from_millis(30));

    // Wait for the worker to finish before dropping `boxed`.
    handle.join().unwrap();

    // Now it is safe to use and then drop `boxed` again.
    println!("owner: final value = {}", boxed);
}
