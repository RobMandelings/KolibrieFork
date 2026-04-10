struct Event {
    id: u64,
}

struct Struct2<'a> {
    slice: &'a [Event],
}

struct Struct1 {
    events: Vec<Event>,
}

impl Struct1 {
    fn new(events: Vec<Event>) -> Self {
        Self { events }
    }

    // Create a Struct2 that borrows from self.events
    fn window<'a>(&'a self, range: std::ops::Range<usize>) -> Struct2<'a> {
        Struct2 {
            slice: &self.events[range],
        }
    }
}

fn main() {
    let s1 = Struct1::new(vec![
        Event { id: 1 },
        Event { id: 2 },
        Event { id: 3 },
        Event { id: 4 },
    ]);

    // Struct2 borrows from Struct1; Struct1 owns Vec<Event>.
    let w1 = s1.window(0..2);
    let w2 = s1.window(1..4);

    // Use the windows while s1 is still alive.
    for e in w1.slice {
        println!("w1: {}", e.id);
    }
    for e in w2.slice {
        println!("w2: {}", e.id);
    }
}