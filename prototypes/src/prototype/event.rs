#[derive(Clone)]
pub struct Event<I> {
    pub ts: Time,
    pub payload: I, // Heap allocated data: handle is cheap to clone, but the data is not
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