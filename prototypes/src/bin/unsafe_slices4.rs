use std::slice;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

// Newtype wrapper around a raw pointer.
// Raw pointers are not Send by default, so we wrap it.
#[derive(Copy, Clone)]
struct SendPtr<T>(*const T);

// SAFETY:
// We promise that sending this pointer between threads is okay in this program
// because the backing Vec stays alive until both threads are joined, and the
// threads only read from it.
unsafe impl<T> Send for SendPtr<T> {}

struct UnsafeSlice<T> {
    ptr: SendPtr<T>,
    len: usize,
}

impl<T> UnsafeSlice<T> {

    pub fn new(ptr: SendPtr<T>, len: usize) -> Self {
        Self {
            ptr,
            len
        }
    }

    pub fn get_slice(&self) -> &[T] {
        unsafe {slice::from_raw_parts(self.ptr.0, self.len)}
    }
}

fn main() {
    let ids: Vec<i32> = (0..10).collect();

    let len1 = ids.len() - 2; // slice from index 2 to end
    let len2 = ids.len() - 5; // slice from index 5 to end

    let ptr1 = SendPtr(unsafe { ids.as_ptr().add(2) });
    let ptr2 = SendPtr(unsafe { ids.as_ptr().add(5) });

    let (tx1, rx1) = mpsc::channel::<(UnsafeSlice<i32>)>();
    let (tx2, rx2) = mpsc::channel::<(UnsafeSlice<i32>)>();

    let h1 = thread::spawn(move || {
        let u_slice = rx1.recv().unwrap();

        // SAFETY:
        // - ptr.0 points to ids[2]
        // - len = ids.len() - 2, so the slice reaches exactly to the end
        // - ids is still alive while this thread runs
        let slice: &[i32] = u_slice.get_slice();

        println!("thread 1 slice: {:?}", slice);
        for x in slice {
            println!("thread 1 saw {x}");
        }
    });

    let h2 = thread::spawn(move || {
        let u_slice = rx2.recv().unwrap();

        // SAFETY:
        // - ptr.0 points to ids[5]
        // - len = ids.len() - 5, so the slice reaches exactly to the end
        // - ids is still alive while this thread runs
        let slice = u_slice.get_slice();
        sleep(Duration::from_secs(2));

        println!("thread 2 slice: {:?}", slice);
        for x in slice {
            println!("thread 2 saw {x}");
        }
    });

    tx1.send(UnsafeSlice::new(ptr1, len1)).unwrap();
    tx2.send(UnsafeSlice::new(ptr2, len2)).unwrap();
    println!("Main thread execution!");

    h1.join().unwrap();
    h2.join().unwrap();

    // ids is dropped only after both threads are done
}