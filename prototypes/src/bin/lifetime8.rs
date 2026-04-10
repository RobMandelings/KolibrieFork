struct Event {
    id: u64,
    // ...
}

struct WindowMeta {
    start: usize,
    end:   usize, // exclusive
}

// This is your sliding-window state.
struct WindowState {
    events: Vec<Event>,
    windows: Vec<WindowMeta>, // multiple active windows
}

impl WindowState {
    fn new(events: Vec<Event>) -> Self {
        Self { events, windows: Vec::new() }
    }

    fn add_window(&mut self, start: usize, end: usize) {
        assert!(start <= end && end <= self.events.len());
        self.windows.push(WindowMeta { start, end });
    }

    fn window_slice(&self, w: &WindowMeta) -> &[Event] {
        &self.events[w.start..w.end]
    }
}

fn main() {
    let mut state = WindowState::new(vec![
        Event { id: 1 },
        Event { id: 2 },
        Event { id: 3 },
        Event { id: 4 },
    ]);

    // multiple active windows, no cloning
    state.add_window(0, 2);
    state.add_window(1, 4);

    for w in &state.windows {
        let slice = state.window_slice(w);
        println!("{:?}", slice.iter().map(|e| e.id).collect::<Vec<_>>());
    }
}