use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct SendPtr<T>(*mut T);
unsafe impl<T> Send for SendPtr<T> {}

// Simple "done" message; we just use unit plus an ID.
struct Done {
    id: usize,
}

fn main() {
    let mut boxed = Box::new(0usize);
    let job_id = 1usize;

    // Channels:
    // owner -> worker A
    let (tx_a, rx_a) = mpsc::channel::<(usize, SendPtr<usize>)>();
    // worker A -> worker B
    let (tx_b, rx_b) = mpsc::channel::<(usize, SendPtr<usize>)>();
    // worker B -> owner
    let (tx_done, rx_done) = mpsc::channel::<Done>();

    // Worker A: receives job, mutates, forwards to B.
    let worker_a = thread::spawn(move || {
        while let Ok((id, ptr)) = rx_a.recv() {
            unsafe {
                for _ in 0..5 {
                    *ptr.0 += 1;
                    println!("[A] id {id}, value = {}", *ptr.0);
                    thread::sleep(Duration::from_millis(5));
                }
            }
            // forward to worker B
            if tx_b.send((id, ptr)).is_err() {
                break;
            }
        }
        println!("[A] exiting");
    });

    // Worker B: receives job, mutates, then signals "done" to owner.
    let worker_b = thread::spawn(move || {
        while let Ok((id, ptr)) = rx_b.recv() {
            unsafe {
                for _ in 0..5 {
                    *ptr.0 += 1;
                    println!("[B] id {id}, value = {}", *ptr.0);
                    thread::sleep(Duration::from_millis(5));
                }
            }
            // Now we are done with this pointer; tell the owner.
            if tx_done.send(Done { id }).is_err() {
                break;
            }
            // After this, B does not touch ptr.0 anymore.
        }
        println!("[B] exiting");
    });

    // OWNER:

    // Create job pointing to boxed and send to worker A.
    let ptr = SendPtr(&mut *boxed as *mut usize);
    tx_a.send((job_id, ptr)).unwrap();

    // Wait until worker B signals completion for this job.
    let done = rx_done.recv().unwrap();
    assert_eq!(done.id, job_id);

    // At this point, both workers have finished touching `boxed`.
    println!("[owner] final value = {}", boxed);

    // For this demo, shut workers down by closing channels.
    drop(tx_a);
    // You don't need to drop tx_b and tx_done: when the sender is dropped, the receiver will stop receiving and the worker thread continues
    // E.g. sender tx_a is dropped, so the worker_a will stop receiving, so worker_a drops. Since worker_a owns the sender for worker_b, worker_b will stop receiving, which means that
    // tx_done will also be dropped, which means that the owner will stop receiving (so rx_done.recv() will continue instead of block) which means that the owning thread will also stop.
    // drop(tx_b);
    // drop(tx_done);

    worker_a.join().unwrap();
    worker_b.join().unwrap();
}