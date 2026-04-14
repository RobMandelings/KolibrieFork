use std::sync::{
    mpsc, Arc, Condvar, Mutex,
};
use std::{slice, thread};

// Newtype wrapper around a raw pointer.
// Raw pointers are not Send by default, so we wrap it.
#[derive(Copy, Clone)]
struct SendPtr<T>(*const T);

// SAFETY:
// We promise that sending this pointer between threads is okay in this program
// because the backing Vec stays alive until both threads are joined, and the
// threads only read from it.
unsafe impl<T> Send for SendPtr<T> {}

#[derive(Copy, Clone)]
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

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    tx: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));

        for _ in 0..num_workers {
            let rx = Arc::clone(&rx);
            thread::spawn(move || loop {
                let job = {
                    let guard = rx.lock().unwrap();
                    guard.recv()
                };

                match job {
                    Ok(job) => job(),
                    Err(_) => break, // sender dropped => shutdown
                }
            });
        }

        Self { tx }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Box::new(f)).unwrap();
    }

    pub fn submit_with_group<F>(&self, group: &TaskGroup, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        group.add_task();

        let group = group.clone();
        self.tx.send(Box::new(move || {
            f();
            group.done();
        })).unwrap();
    }
}

#[derive(Clone)]
pub struct TaskGroup {
    inner: Arc<TaskGroupInner>,
}

struct TaskGroupInner {
    state: Mutex<usize>, // number of unfinished tasks
    cv: Condvar,
}

impl TaskGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TaskGroupInner {
                state: Mutex::new(0),
                cv: Condvar::new(),
            }),
        }
    }

    pub fn add_task(&self) {
        let mut count = self.inner.state.lock().unwrap();
        *count += 1;
    }

    pub fn done(&self) {
        let mut count = self.inner.state.lock().unwrap();
        *count -= 1;

        if *count == 0 {
            self.inner.cv.notify_all();
        }
    }

    pub fn wait(&self) {
        let mut count = self.inner.state.lock().unwrap();

        while *count > 0 {
            count = self.inner.cv.wait(count).unwrap();
        }
    }
}

fn main() {
    let ids: Vec<i32> = (0..10).collect();

    let len1 = ids.len() - 2; // slice from index 2 to end
    let len2 = ids.len() - 5; // slice from index 5 to end

    let ptr1 = SendPtr(unsafe { ids.as_ptr().add(2) });
    let ptr2 = SendPtr(unsafe { ids.as_ptr().add(5) });

    let u_slice1 = UnsafeSlice::new(ptr1, len1);
    let u_slice2 = UnsafeSlice::new(ptr2, len2);

    let pool = ThreadPool::new(4);

    // Parent thread/request A
    let group_a = TaskGroup::new();
    for i in 0..5 {
        pool.submit_with_group(&group_a, move || {
            println!("A: task {i} running on worker");
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let slice = u_slice1.get_slice();
            println!("From slice 1: {:?}", slice);
        });
    }

    // Wait only for A's tasks
    group_a.wait();
    println!("Parent A can continue now");

    // Parent thread/request B
    let group_b = TaskGroup::new();
    for i in 0..3 {
        pool.submit_with_group(&group_b, move || {
            println!("B: task {i} running on worker");
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let slice = u_slice2.get_slice();
            println!("From slice 2: {:?}", slice);
        });
    }

    // B may still be running
    group_b.wait();
    println!("Parent B can continue now");
}