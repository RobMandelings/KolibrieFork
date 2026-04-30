use serde::{Deserialize, Serialize};
use crate::prototype::event::Time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowParams {
    pub size: Time,
    pub slide: Time,
    pub offset: Time
}

impl WindowParams {
    pub fn new(size: Time, slide: Time, offset: Time) -> WindowParams {
        WindowParams {
            size,
            slide,
            offset
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2RWindowConfig {
    pub window_iri: String,
    pub window_params: WindowParams,
}