use std::sync::Once;
use env_logger;

static INIT: Once = Once::new();

pub fn init_logging() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}