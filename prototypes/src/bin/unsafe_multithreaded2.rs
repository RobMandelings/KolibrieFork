use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct SendPtr<T>(*mut T);
unsafe impl<T> Send for SendPtr<T> {}

fn main() {
    let mut boxed = Box::new(0usize);

    // Channel to send jobs (raw-pointer wrappers) to the worker.
    let (tx, rx) = mpsc::channel::<SendPtr<usize>>();

    // Fixed worker thread.
    let worker = thread::spawn(move || {
        // Loop until channel is closed.
        while let Ok(ptr) = rx.recv() {
            unsafe {
                // Do whatever work you want on the pointee.
                for _ in 0..10 {
                    *ptr.0 += 1;
                    println!("worker: value = {}", *ptr.0);
                    thread::sleep(Duration::from_millis(10));
                }
            }
            // After this iteration, worker stops using ptr.0.
        }
        println!("worker: channel closed, exiting");
    });

    // OWNER THREAD:

    // Create a job pointing to `boxed` and send it.
    let ptr = SendPtr(&mut *boxed as *mut usize);
    tx.send(ptr).unwrap();

    // Wait for some time (in a real design, you’d wait for an explicit
    // "done" signal instead of sleeping).
    thread::sleep(Duration::from_millis(200));

    // Close the channel so worker can exit.
    drop(tx);

    worker.join().unwrap();

    println!("owner: final value = {}", boxed);
}