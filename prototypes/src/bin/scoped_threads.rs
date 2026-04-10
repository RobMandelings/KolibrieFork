use std::thread;

fn main() {
    let mut numbers = vec![1, 2, 3, 4];

    thread::scope(|s| {
        // Immutable borrow in one thread
        s.spawn(|| {
            for n in &numbers {
                println!("seen: {n}");
            }
        });

        // Immutable borrow in another thread
        s.spawn(|| {
            for n in &numbers {
                println!("seen: {n}");
            }
        });
        // Both threads can use `numbers` by reference; no Arc needed.
    });

    // All scoped threads are done here
    println!("after: {:?}", numbers);
}
