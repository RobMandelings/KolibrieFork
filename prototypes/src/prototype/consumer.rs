use RSPPrototype::prototype::event::Event;
use crate::helpers::event;

pub struct Consumer {
    consume_fn: Box<dyn Fn(Vec<&Event>)>,
}

impl Consumer {

    pub(crate) fn new(f: impl Fn(Vec<&Event>) + 'static) -> Self {
        Self {
            consume_fn: Box::new(f),
        }
    }

    pub fn consume(&self, events: Vec<&Event>) {
        (self.consume_fn)(events);
    }
}

#[test]
fn consumer_calls_closure() {

    let c = Consumer::new(|events: Vec<&Event>| {
        assert!(!events.is_empty());
        // capture &mut state via interior mutability if needed (e.g. Cell/RefCell)
        println!("got {} events", events.len());
    });

    let e = event(5);
    c.consume(vec![&e]);
}