#[cfg(test)]
mod perf_tests {
    use std::rc::Rc;
    use std::time::Instant;
    use crate::prototype::event::Time;
    use crate::prototype::helpers::{event, wc, wc_struct};
    use crate::prototype::slide_strategy::WindowSnapshotStrategy;
    use crate::{CloneStrategy, Event, ExpireStrategy, RcStrategy, SlidingWindowOperator, WindowParams};
    use crate::prototype::slide_strategy::clone_strategy::CloneContainer;
    use crate::prototype::slide_strategy::expire_strategy::SliceContainer;
    use crate::prototype::slide_strategy::rc_strategy::RcContainer;

    fn run_throughput_test<S>(name: &str, mut op: SlidingWindowOperator<String, S>)
    where
        S: WindowSnapshotStrategy<String>,
    {
        let n_events: usize = 1_000_000;
        let start_ts: Time = 0;

        let start = Instant::now();

        for i in 0..n_events {
            let ts: Time = start_ts + (i as Time);
            op.event_arrives(event(ts));
        }

        let elapsed = start.elapsed();
        let secs = elapsed.as_secs_f64();
        let throughput = (n_events as f64) / secs;

        eprintln!(
            "[{}] Processed {} events in {:.3} s ⇒ {:.2} events/s",
            name, n_events, secs, throughput
        );
    }

    #[test]
    fn sliding_window_operator_throughput_smoke_test() {

        // Simple configuration: non-overlapping windows of size 10.
        let config = wc(10, 10, 0);
        let config_iri = config.window_iri.clone();
        // Strategy 1: ExpireStrategy with borrowed events
        let consume_expire = Box::new(|_events: SliceContainer<String>| {});
        let expire_strat = ExpireStrategy::new();
        let mut op_expire = SlidingWindowOperator::single_window(config.clone(), expire_strat);
        op_expire.add_consumer(&config_iri, consume_expire);

        // Strategy 2: CloneStrategy with owned/cloned events
        let consume_clone = Box::new(|_events: CloneContainer<String>| {});
        let clone_strat = CloneStrategy::new();
        let mut op_clone = SlidingWindowOperator::single_window(config.clone(), clone_strat);
        op_clone.add_consumer(&config_iri, consume_clone);

        let rc_consume = Box::new(|_events: RcContainer<String>| {});
        let rc_strat = RcStrategy::new();
        let mut op_ref_count = SlidingWindowOperator::single_window(config.clone(), rc_strat);
        op_ref_count.add_consumer(&config_iri, rc_consume);

        run_throughput_test("ExpireStrategy", op_expire);
        run_throughput_test("CloneStrategy", op_clone);
        run_throughput_test("RefCountStrategy", op_ref_count);
    }
}