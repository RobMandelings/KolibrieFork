use std::thread;

static X: i32 = 5;              // lives for entire program, `'static`

fn main() {
    let r: &'static i32 = &X;   // r: &'static i32, and i32: Sync ⇒ &i32: Send

    thread::spawn(move || {
        // ok: closure is Send + 'static
        println!("{r}");
    })
        .join()
        .unwrap();
}