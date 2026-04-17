use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;
use crate::{Event, WindowParams};
use crate::prototype::event::Time;
use crate::prototype::window_params::S2RWindowConfig;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn new_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}





/// Creates a window config with UUID for quick testing
pub fn wc(size: Time, slide: Time, offset: Time) -> S2RWindowConfig {
    S2RWindowConfig {
        window_iri: format!("urn:window:{}", Uuid::new_v4()),
        window_params: WindowParams { size, slide, offset },
    }
}

pub fn wc_struct(params: WindowParams) -> S2RWindowConfig {
    let WindowParams { size, slide, offset } = params;
    wc(size, slide, offset)
}