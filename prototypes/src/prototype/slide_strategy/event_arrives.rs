use crate::Event;

pub trait EventArrives<I> {
    fn event_arrives(&mut self, event: Event<I>);
}