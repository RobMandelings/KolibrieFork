use crate::WindowParams;

// Global, immutable configuration
pub static WINDOW_CONFIGS: &[WindowParams] = &[
    WindowParams { size: 1,   slide: 1, offset: 0 },
    WindowParams { size: 4,   slide: 1, offset: 0 },
    WindowParams { size: 16,  slide: 1, offset: 0 },
    WindowParams { size: 32,  slide: 1, offset: 0 },
    WindowParams { size: 64,  slide: 1, offset: 0 },
    WindowParams { size: 128, slide: 1, offset: 0 },
];