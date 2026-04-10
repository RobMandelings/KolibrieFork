use std::collections::HashMap;
use prototypes::{event, Event};

struct Result<'a> {
    solution_mapping: HashMap<String, &'a String>
}

impl<'a> Result<'a> {

    pub fn new() -> Self {
        Result {
            solution_mapping: HashMap::new()
        }
    }

    pub fn add_pair(mut self, str: String, content: &'a String) -> Self {
        self.solution_mapping.insert(str, content);
        self
    }
}

fn get_str(slice: &[Event<String>]) -> &String {
    &slice[0].payload
}

fn main() {
    let vec: Vec<Event<String>> = vec![
        // Event::new(0, 0, "Hallo".to_string()),
        event(0),
        event(1),
        event(2),
    ];
    let slice = &vec[0..1];

    // This extracts the string from that slices and returns it
    let first_str = get_str(slice);
    let result = Result::new().add_pair("bam".to_string(), first_str);

    println!("{first_str}");
}
