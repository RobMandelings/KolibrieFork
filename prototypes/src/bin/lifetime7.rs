struct Event {
    id: u64,
}

struct Struct2<'a> {
    slice: &'a [Event],
}

struct Struct1<'a> {
    events: Vec<Event>,
    window: Option<Struct2<'a>>,
}

impl<'a> Struct1<'a> {
    fn new(events: Vec<Event>) -> Self {
        Self { events, window: None }
    }

    fn init_window(&mut self) {
        // try to create a Struct2 borrowing from self.events
        // let slice = &self.events[..];
        // self.window = Some(Struct2 { slice });
    }
}

fn main() {

}