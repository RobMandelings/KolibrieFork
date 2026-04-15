use std::thread;

fn main() {
    let x: i32 = 5;             // i32: Sync, so &i32: Send
    let r: &i32 = &x;           // r: &i32, but lifetime tied to main (non-'static)

    thread::spawn(move || {
        // error: `r` has non-`'static` lifetime, cannot be moved into thread
        println!("{r}");
    });
}