use serde::{Deserialize, Serialize};
use crate::prototype::event::Time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowParams {
    pub size: Time,
    pub slide: Time,
    pub offset: Time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2RWindowConfig {
    pub window_iri: String,
    pub window_params: WindowParams,
}