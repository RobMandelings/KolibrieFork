use std::rc::Rc;
use std::sync::Arc;
use crate::Event;
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::arc_strategy::ArcContainer;
use crate::prototype::slide_strategy::clone_strategy::CloneContainer;
use crate::prototype::slide_strategy::rc_strategy::RcContainer;
use crate::prototype::slide_strategy::slice_strategy::{SliceContainer};
use crate::prototype::window_bounds::after_open;

pub fn slice_by_ts<I>(content: &Vec<Event<I>>, open: Time) -> &[Event<I>] {
    let lo = content.partition_point(|e| !after_open(&open, &e.ts));
    &content[lo..]
}

pub fn create_slice_report<I>(content: &Vec<Event<I>>) -> SliceContainer<I> {
    SliceContainer(&content)
}

pub fn create_arc_report<I>(content: &Vec<Arc<Event<I>>>) -> ArcContainer<I> {
    ArcContainer(content.clone())
}

pub fn create_rc_report<I>(content: &Vec<Rc<Event<I>>>) -> RcContainer<I> {
    RcContainer(content.clone())
}

pub fn create_clone_report<I: Clone>(content: &Vec<Event<I>>, open_time: Time) -> CloneContainer<I> {
    CloneContainer(content.clone())
}