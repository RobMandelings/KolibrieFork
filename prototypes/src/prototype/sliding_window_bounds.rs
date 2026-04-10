use crate::prototype::event::Time;
use crate::prototype::window_bounds::WindowBounds;
use crate::prototype::window_params::S2RWindowConfig;
use crate::WindowParams;

#[derive(Debug, Clone)]
pub struct SlidingWindowBounds {
    pub active_bounds: WindowBounds,
    window_params: WindowParams,
    window_iri: String,
}

fn compute_bounds(params: &WindowParams, idx: usize) -> WindowBounds {
    let n = idx as Time;
    let open  = params.offset + n * params.slide;       // o_n = t0 + nβ
    let close = params.offset + params.size + n * params.slide;   // c_n = t0 + ω + nβ

    WindowBounds {open, close}
}

/// Computes the smallest cutoff value for the stream section that is kept in memory
pub fn compute_earliest_open_time<'a, I>(iter: I) -> Time
where
    I: IntoIterator<Item = &'a SlidingWindowBounds>,
{
    iter.into_iter().map(|b| &b.active_bounds).map(|a| a.open).min().unwrap()
}

impl SlidingWindowBounds {

    pub fn new(window_config: S2RWindowConfig) -> Self {
        let bounds = compute_bounds(&window_config.window_params, 0);

        Self {
            active_bounds: bounds,
            window_params: window_config.window_params,
            window_iri: window_config.window_iri
        }
    }

    /// Checks whether the window will slide if you update with given timestamp
    pub fn slides_at(&self, ts: Time) -> bool {
        // If the current active window closes, then the active window changes (= slide)
        self.active_bounds.after_close(ts)
    }

    /// Relates to the 'scope' dimension in SECRET (where does the query apply)
    fn get_active_window_idx(&self, t: Time) -> usize {
        let WindowParams { size, slide, offset } = self.window_params;

        if slide == 0 {
            panic!("slide cannot be 0");
        }

        // If t is strictly before the first window opens, no active window.
        if t < offset {
            panic!("Timestamp should be after offset");
        }

        // n = max(0, ceil((t - t0 - w) / beta))
        let num   = t.saturating_sub(offset).saturating_sub(size); // t - t0 - w
        let n_raw = (num + slide - 1) / slide;                // ceil(num / beta) for integers
        let n     = n_raw.max(0);                           // max(0, ...); Time is unsigned so this is just n_raw
        n as usize
    }

    /// Checks what the new active window should be and updates the bounds
    pub fn slide(&mut self, ts: Time) {
        let active_window_idx = self.get_active_window_idx(ts);

        // Update bounds to reflect new active window
        self.active_bounds = compute_bounds(&self.window_params, active_window_idx);
    }
}

#[cfg(test)]
mod earliest_open_tests {
    use crate::prototype::helpers::wc;
    use super::*; // SlidingWindowBounds, WindowParams, Time, compute_earliest_open_time

    fn wb(size: Time, slide: Time, offset: Time) -> SlidingWindowBounds {
        // Helper: construct bounds with given active.open; adjust to your actual API
        SlidingWindowBounds::new(wc(size, slide, offset))
    }

    #[test]
    fn earliest_open_single_window() {
        let bounds = vec![wb(10, 10, 5)];
        let earliest = compute_earliest_open_time(&bounds);
        assert_eq!(earliest, 5);
    }

    #[test]
    fn earliest_open_multiple_windows() {
        let bounds = vec![
            wb(10, 10, 10),
            wb(10, 10, 0),
            wb(10, 10, 20),
        ];
        let earliest = compute_earliest_open_time(&bounds);
        assert_eq!(earliest, 0);
    }

    #[test]
    fn earliest_open_all_equal() {
        let bounds = vec![
            wb(10, 10, 7),
            wb(10, 10, 7),
            wb(10, 10, 7),
        ];
        let earliest = compute_earliest_open_time(&bounds);
        assert_eq!(earliest, 7);
    }

    #[test]
    fn earliest_open_updates_after_slide() {
        // Two windows, second starts later.
        let b1 = wb(10, 10, 0);
        let b2 = wb(10, 10, 5);

        let mut vec = vec![b1, b2];

        // Initial earliest should be the min of their opens.
        let earliest0 = compute_earliest_open_time(&vec);
        assert_eq!(earliest0, 0);

        // Slide first window forward so its open becomes greater than the second's.
        vec[0].slide(100); // pick a ts that advances window 0 significantly

        let earliest1 = compute_earliest_open_time(&vec);
        assert_eq!(earliest1, 5);
    }
}

#[cfg(test)]
mod sliding_window_bounds_tests {
    use crate::prototype::helpers::wc;
    use super::*;

    #[test]
    fn new_initializes_active_bounds_for_first_window() {
        let p = wc(10, 5, 0);
        let swb = SlidingWindowBounds::new(p);

        assert_eq!(swb.active_bounds.open, 0);
        assert_eq!(swb.active_bounds.close, 10);
    }

    #[test]
    fn slides_at_is_false_before_close_true_at_and_after_close() {
        let p = wc(10, 10, 0);
        let swb = SlidingWindowBounds::new(p); // W0: (0,10]

        assert!(!swb.slides_at(0));
        assert!(!swb.slides_at(9));
        assert!(!swb.slides_at(10)); // at c_0 per (o_i, c_i]
        assert!(swb.slides_at(11));
    }

    #[test]
    fn get_active_window_idx_matches_secret_definition() {
        let p = wc(10, 10, 0);
        let swb = SlidingWindowBounds::new(p);

        // W0 closes at 10, W1 at 20, etc.
        assert_eq!(swb.get_active_window_idx(0), 0);
        assert_eq!(swb.get_active_window_idx(5), 0);
        assert_eq!(swb.get_active_window_idx(10), 0);

        assert_eq!(swb.get_active_window_idx(11), 1);
        assert_eq!(swb.get_active_window_idx(15), 1);
        assert_eq!(swb.get_active_window_idx(20), 1);

        assert_eq!(swb.get_active_window_idx(21), 2);
    }

    #[test]
    fn slide_updates_active_bounds_to_correct_window() {
        let p = wc(10, 10, 0);
        let mut swb = SlidingWindowBounds::new(p);

        // Initially W0: (0,10]
        assert_eq!(swb.active_bounds.open, 0);
        assert_eq!(swb.active_bounds.close, 10);

        swb.slide(5); // still W0
        assert_eq!(swb.active_bounds.open, 0);
        assert_eq!(swb.active_bounds.close, 10);

        swb.slide(11); // move to W1: (10,20]
        assert_eq!(swb.active_bounds.open, 10);
        assert_eq!(swb.active_bounds.close, 20);

        swb.slide(25); // move to W2: (20,30]
        assert_eq!(swb.active_bounds.open, 20);
        assert_eq!(swb.active_bounds.close, 30);
    }

    #[test]
    #[should_panic(expected = "slide cannot be 0")]
    fn get_active_window_idx_panics_on_zero_slide() {
        let p = wc(10, 0, 0);
        let swb = SlidingWindowBounds::new(p);

        let _ = swb.get_active_window_idx(10);
    }

    #[test]
    #[should_panic(expected = "Timestamp should be after offset")]
    fn get_active_window_idx_panics_if_t_before_offset() {
        let p = wc(10, 10, 5);
        let swb = SlidingWindowBounds::new(p);

        let _ = swb.get_active_window_idx(4);
    }
}