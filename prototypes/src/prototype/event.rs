#[derive(Clone)]
pub struct Event<I> {
    pub ts: Time,
    pub payload: I, // Heap allocated data: handle is cheap to clone, but the data is not
}

// Concrete event type with heap-allocated bytes
pub type ByteEvent = Event<Box<[u8]>>;

pub fn make_byte_event(ts: Time, size: usize) -> Event<Box<[u8]>> {
    Event {
        ts,
        payload: vec![0u8; size].into_boxed_slice(),
    }
}

fn make_string_payload(len: usize) -> String {
    // All zeroes, or 'x', doesn't matter as long as length is `len`.
    "0".repeat(len)
}

pub fn make_string_event(ts: Time) -> Event<String> {
    Event::new(ts, make_string_payload(1000))
}

pub fn make_copy_event(ts: Time) -> Event<u64> {
    Event::new(ts, 0)
}

impl<I> Event<I> {

    pub fn new(ts: Time, payload: I) -> Event<I>  {
        Event {
            ts,
            payload
        }
    }

}

pub type Time = u64;