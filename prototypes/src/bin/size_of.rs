use std::rc::Rc;
use prototypes::prototype::event::Triple;

fn main() {
    let t = Rc::new(Triple { subject: 0, predicate: 0, object: 0 });
    println!("size_of::<Rc<Triple>>() = {}", size_of::<Rc<Triple>>());
    // just to silence unused variable warning
    println!("Address of t (for sanity) = {:p}", t);
}