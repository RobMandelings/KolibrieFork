#[derive(Debug)]
struct Event {
    id: u32,
}

fn recurse(mut events: Vec<Event>) {
    // Take one event out; this event will be dropped when this stack frame ends.
    if let Some(current) = events.pop() {
        println!("In this frame: events = {:?}, current = {:?}", events, current);

        // Pass a reference to the current event downwards.
        recurse_inner(&current, events);
        // After recurse_inner returns, `current` is still alive here.

        // `current` is dropped here, when this frame returns.
    } else {
        println!("No more events");
    }
}

fn recurse_inner(current: &Event, mut events: Vec<Event>) {
    println!("  Inner: events = {:?}, current = {:?}", events, current);

    if let Some(next) = events.pop() {
        // Recurse deeper with a new "current"
        recurse_inner(&next, events);
        // `next` is dropped here
    }
    // `current` is still valid for the whole duration of this function.
}

fn main() {
    let events = vec![
        Event { id: 1 },
        Event { id: 2 },
        Event { id: 3 },
    ];

    recurse(events);
}
