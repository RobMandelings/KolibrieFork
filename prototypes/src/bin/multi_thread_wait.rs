use std::sync::{
    mpsc, Arc, Condvar, Mutex,
};
use std::thread;

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
    let pool = ThreadPool::new(4);

    // Parent thread/request A
    let group_a = TaskGroup::new();
    for i in 0..5 {
        pool.submit_with_group(&group_a, move || {
            println!("A: task {i} running on worker");
            std::thread::sleep(std::time::Duration::from_millis(1000));
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
        });
    }


    // B may still be running
    group_b.wait();
    println!("Parent B can continue now");
}