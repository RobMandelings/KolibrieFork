struct Event {
    id: u64,
}

struct Window<'a> {
    events: &'a [Event],
}

impl<'a> Window<'a> {
    fn first_id(&self) -> Option<u64> {
        self.events.first().map(|e| e.id)
    }
}

fn main() {
    let events = vec![
        Event { id: 1 },
        Event { id: 2 },
        Event { id: 3 },
    ];

    // Window just *borrows* from an existing slice.
    let w = Window { events: &events[0..2] };

    println!("{:?}", w.first_id());
}