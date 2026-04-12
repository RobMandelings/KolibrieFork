#[derive(Clone)]
pub struct Event<I> {
    pub ts: Time,
    pub payload: I, // Heap allocated data: handle is cheap to clone, but the data is not
}

// Concrete event type with heap-allocated bytes
pub type ByteEvent = Event<Box<[u8]>>;

pub fn make_byte_event(ts: Time, size: usize) -> ByteEvent {
    Event {
        ts,
        payload: vec![0u8; size].into_boxed_slice(),
    }
}

impl<I> Event<I> {

    pub fn new(ts: Time, payload: I) -> Event<I> {
        Event {
            ts,
            payload
        }
    }

}

pub type Time = u64;