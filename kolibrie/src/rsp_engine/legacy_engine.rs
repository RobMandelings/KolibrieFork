use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use crossbeam::channel::RecvTimeoutError;
use log::debug;
use prototypes::WindowSnapshotStrategy;
use shared::query::{Fallback, SyncPolicy};
use crate::rsp_engine::{emit_results, OperationMode, RSPEngine, WindowResult};

/// All functinality related to legacy window implementations go here
impl<I, O, S> RSPEngine<I, O, S>
where
    O: Clone + Hash + Eq + Send + 'static + From<Vec<(String, String)>>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    S: WindowSnapshotStrategy<I>,
{

    /// Add data to appropriate window based on stream IRI
    pub fn add_to_stream(&mut self, stream_iri: &str, event_item: I, ts: usize) {
        if matches!(self.operation_mode, OperationMode::SingleThread)
            && (self.windows.len() > 1 || self.rsp_query_plan.static_data_plan.is_some())
        {
            self.process_single_thread_window_results();
        }

        let input_norm = Self::normalize_stream_iri(stream_iri);

        // Find windows that match this stream IRI and add the event to these windows
        for (window_idx, window_config) in self.window_configs.iter().enumerate() {
            // Variable stream (e.g. `?s`) matches any stream.
            if window_config.stream_iri.starts_with('?') {
                if let Some(window) = self.windows.get_mut(window_idx) {
                    window.add_to_window(event_item.clone(), ts);
                }
                continue;
            }

            let cfg_norm = Self::normalize_stream_iri(&window_config.stream_iri);
            if cfg_norm == input_norm {
                if let Some(window) = self.windows.get_mut(window_idx) {
                    window.add_to_window(event_item.clone(), ts);
                }
            }
        }
    }

    pub fn process_single_thread_window_results(&mut self)
    where
        O: From<Vec<(String, String)>>,
    {
        // The consumer function that will be called. We clone to get ownership, but it points to the same function
        let consumer = self.r2s_consumer.clone();
        let num_windows = self.windows.len();
        let sync_policy = self.sync_policy.clone();

        // Drain all pending channel results; update last_materialized with replace semantics.
        let mut last_mat = self.single_thread_last_materialized.lock().unwrap();
        let mut had_new_results = false;
        let mut max_ts: usize = 0;

        // Non-blocking drain of the channel: receive all results
        while let Ok(window_result) = self.window_result_receiver.try_recv() {
            max_ts = max_ts.max(window_result.timestamp);
            last_mat.insert(window_result.window_iri.clone(), window_result.results);
            had_new_results = true;
        }

        if !had_new_results {
            return;
        }

        // Check whether to emit based on policy.
        // Only emit when all windows have been materialized
        if last_mat.len() == num_windows {
            debug!(
                "SingleThread: all {} windows materialized, emitting",
                num_windows
            );
            let static_data_plan = self.rsp_query_plan.static_data_plan.clone();
            emit_results(
                &*last_mat,
                &static_data_plan,
                &self.static_db,
                &self.r2s_operator,
                max_ts,
                &consumer,
            );

            match sync_policy {
                // Wait: require all windows to fire again before next emission.
                // Timeout: no wall-clock timer in single-threaded context; treat as Wait.
                SyncPolicy::Wait | SyncPolicy::Timeout { .. } => {
                    last_mat.clear();
                }
                // Steal: keep last_mat so stale data from non-firing windows is reused.
                SyncPolicy::Steal => {}
            }
        } else {
            debug!(
                "SingleThread: waiting for more windows ({}/{})",
                last_mat.len(),
                num_windows
            );
        }
    }

    /// Legacy method for backward compatibility
    pub fn legacy_add(&mut self, event_item: I, ts: usize) {
        // Add to all windows (for backward compatibility)
        for window in &mut self.windows {
            window.add_to_window(event_item.clone(), ts);
        }
    }

    pub fn stop(&mut self) {
        for window in &mut self.windows {
            window.flush();
            window.stop();
        }
        if matches!(self.operation_mode, OperationMode::SingleThread) {
            self.process_single_thread_window_results();
        }
    }

    /// Start a coordinator thread that collects and joins results from multiple windows
    /// (and optionally joins with static background data), respecting `sync_policy`.
    pub(crate) fn start_cross_window_coordinator(&self)
    where
        O: From<Vec<(String, String)>>,
    {
        let receiver = self.window_result_receiver.clone();
        let consumer = self.r2s_consumer.clone();
        let num_windows = self.windows.len();
        let static_data_plan = self.rsp_query_plan.static_data_plan.clone();
        let static_db = self.static_db.clone();
        let sync_policy = self.sync_policy.clone();
        let r2s_operator = Arc::clone(&self.r2s_operator);

        thread::spawn(move || {
            // Latest results per window (replace semantics)
            let mut last_materialized: HashMap<String, Vec<HashMap<String, String>>> =
                HashMap::new();
            // Windows that have fired since the last reset
            let mut cycle_triggered: HashSet<String> = HashSet::new();
            // When the first window fired in the current cycle
            let mut cycle_start: Option<Instant> = None;
            let mut max_ts: usize = 0;

            loop {
                // Compute recv timeout when policy has a finite deadline
                let timeout_remaining = match &sync_policy {
                    SyncPolicy::Timeout { duration, .. } => {
                        cycle_start.map(|start| duration.saturating_sub(start.elapsed()))
                    }
                    _ => None,
                };

                // Receive next window result (or timeout/disconnect)
                let maybe_result: Option<WindowResult> = if let Some(remaining) = timeout_remaining
                {
                    match receiver.recv_timeout(remaining) {
                        Ok(r) => Some(r),
                        Err(RecvTimeoutError::Timeout) => {
                            // Deadline elapsed
                            if !cycle_triggered.is_empty() {
                                match &sync_policy {
                                    SyncPolicy::Timeout {
                                        fallback: Fallback::Steal,
                                        ..
                                    } => {
                                        if last_materialized.len() == num_windows {
                                            emit_results(
                                                &last_materialized,
                                                &static_data_plan,
                                                &static_db,
                                                &r2s_operator,
                                                max_ts,
                                                &consumer,
                                            );
                                        }
                                    }
                                    SyncPolicy::Timeout {
                                        fallback: Fallback::Drop,
                                        ..
                                    } => {
                                        // discard this cycle
                                    }
                                    _ => {}
                                }
                                cycle_triggered.clear();
                                cycle_start = None;
                                max_ts = 0;
                            }
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                } else {
                    match receiver.recv() {
                        Ok(r) => Some(r),
                        Err(_) => break,
                    }
                };

                if let Some(window_result) = maybe_result {
                    debug!(
                        "Coordinator received {} results from window: {}",
                        window_result.results.len(),
                        window_result.window_iri
                    );

                    max_ts = max_ts.max(window_result.timestamp);
                    // Update last_materialized (replace)
                    last_materialized.insert(
                        window_result.window_iri.clone(),
                        window_result.results.clone(),
                    );
                    if cycle_triggered.is_empty() {
                        cycle_start = Some(Instant::now());
                    }
                    cycle_triggered.insert(window_result.window_iri.clone());

                    // Drain any additional pending results
                    while let Ok(wr) = receiver.try_recv() {
                        max_ts = max_ts.max(wr.timestamp);
                        last_materialized.insert(wr.window_iri.clone(), wr.results.clone());
                        cycle_triggered.insert(wr.window_iri.clone());
                    }

                    if cycle_triggered.len() == num_windows {
                        // All windows fired this cycle
                        emit_results(
                            &last_materialized,
                            &static_data_plan,
                            &static_db,
                            &r2s_operator,
                            max_ts,
                            &consumer,
                        );
                        cycle_triggered.clear();
                        cycle_start = None;
                        max_ts = 0;
                    } else {
                        match &sync_policy {
                            SyncPolicy::Steal => {
                                // Emit immediately using stale data from non-firing windows
                                if last_materialized.len() == num_windows {
                                    emit_results(
                                        &last_materialized,
                                        &static_data_plan,
                                        &static_db,
                                        &r2s_operator,
                                        max_ts,
                                        &consumer,
                                    );
                                }
                                cycle_triggered.clear();
                                cycle_start = None;
                                max_ts = 0;
                            }
                            SyncPolicy::Wait | SyncPolicy::Timeout { .. } => {
                                // Keep waiting for remaining windows
                                debug!(
                                    "Coordinator: waiting for more windows ({}/{}) — have: {:?}",
                                    cycle_triggered.len(),
                                    num_windows,
                                    cycle_triggered.iter().collect::<Vec<_>>()
                                );
                            }
                        }
                    }
                }
            }

            debug!("Coordinator: shutdown complete");
        });
    }
}
