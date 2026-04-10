use crate::prototype::event::Time;

#[derive(Debug, Clone)]
pub struct WindowBounds {
    pub open: Time,
    pub close: Time,
}

/// Separate function both for finding expired events and for checking before_close in WindowBounds
/// To keep interval semantics consistent across the design (open/closed interval...)
pub fn before_open(open_time: &Time, ts: &Time) -> bool {
    ts <= open_time
}

pub fn after_open(open_time: &Time, ts: &Time) -> bool {
    !before_open(open_time, ts)
}

impl WindowBounds {

    pub fn new(open: Time, close: Time) -> Self {
        if open == close {
            panic!("Window has width of 0!");
        }
        if open > close {
            panic!("Window open must be < close");
        }

        Self {
            open,
            close
        }
    }

    /// SECRET paper uses (o_i, c_i] intervals.
    /// At the time c_i, the window closes, but is not closed yet
    /// At time o_i, the window opens, but is not opened yet.
    pub fn within(&self, ts: Time) -> bool {
        !self.before_open(ts) && !self.after_close(ts)
    }

    /// (o_i, c_i] so when ts = o_i, the window is still closed ('opens at o_i')
    pub fn before_open(&self, ts: Time) -> bool {
        before_open(&self.open, &ts)
    }

    /// (o_i, c_i] so when ts = c_i, the window is still open ('closes at c_i')
    pub fn after_close(&self, ts: Time) -> bool {
        ts > self.close
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_bounds_before_open_inclusive_at_open() {
        let bounds = WindowBounds { open: 10, close: 20 };

        assert!(bounds.before_open(0));
        assert!(bounds.before_open(9));
        assert!(bounds.before_open(10)); // ts = o_i => window is opening but not opened

        assert!(!bounds.before_open(11));
        assert!(!bounds.before_open(20));
        assert!(!bounds.before_open(21));
    }

    #[test]
    fn window_bounds_after_close_inclusive_at_close() {
        let bounds = WindowBounds { open: 10, close: 20 };

        assert!(!bounds.after_close(0));
        assert!(!bounds.after_close(10));
        assert!(!bounds.after_close(19));
        assert!(!bounds.after_close(20)); // ts = c_i => window is closing at this time, but not closed yet
        assert!(bounds.after_close(21));
    }

    #[test]
    fn window_bounds_within_matches_open_closed_interval() {
        let bounds = WindowBounds { open: 10, close: 20 };

        // (o_i, c_i] -> 10 < ts <= 20
        assert!(!bounds.within(10)); // equals open: outside
        assert!(bounds.within(11));
        assert!(bounds.within(15));
        assert!(bounds.within(20));  // equals close: inside

        assert!(!bounds.within(0));
        assert!(!bounds.within(9));
        assert!(!bounds.within(21));
    }

    #[test]
    #[should_panic]
    fn window_bounds_trivial_single_point_window() {
        let bounds = WindowBounds::new(10, 10);
    }
}