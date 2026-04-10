mod consumer;
mod helpers;
mod sliding_window_op;
mod sliding_window_bounds;
mod window_bounds;
mod slide_strategy;
mod window_params;

/// Responsible for checking whether window content should be emitted or not.
trait ReportChecker {
    fn should_report(&self) -> bool;
}

/// Keeps track of whether the current active window will close for certain timestamp
/// With Window Close reporting, you should report if the window would close
/// From SECRET model, the active window = the leftmost open window
struct WindowCloseReportChecker {}

impl ReportChecker for WindowCloseReportChecker {
    fn should_report(&self) -> bool {
        todo!()
    }
}

fn main() {}
