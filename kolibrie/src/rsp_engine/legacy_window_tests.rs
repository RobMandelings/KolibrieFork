use std::cell::RefCell;
use std::rc::Rc;
use prototypes::prototype::event::Time;
use prototypes::prototype::helpers::wc;
use prototypes::s2r::{ContentContainer, LegacyWindow, Report, ReportStrategy, Tick};

fn make_payload(ts: Time) -> Time {
    ts
}

fn add(window: &mut LegacyWindow<Time>, ts: Time) {
    window.add_to_window(ts, ts as usize)
}

#[test]
fn legacy_window_reports_on_window_close() {
    let config = wc(10, 10, 0);
    let window_iri = config.window_iri.clone();
    let width = config.window_params.size;
    let slide = config.window_params.slide;

    let reported: Rc<RefCell<Vec<Vec<Time>>>> = Rc::new(RefCell::new(Vec::new()));
    let reported_clone = Rc::clone(&reported);

    let mut report = Report::new();
    report.add(ReportStrategy::OnWindowClose);

    let mut window = LegacyWindow::new(width as usize, slide as usize, report, Tick::TimeDriven, window_iri);

    window.register_callback(Box::new(move |content: ContentContainer<Time>| {
        let mut ts_list: Vec<Time> = content
            .elements
            .iter()
            .copied()
            .collect();
        ts_list.sort(); // sort because HashSet provides no guarantee about order
        reported_clone.borrow_mut().push(ts_list);
    }));

    // First window (0,10]: events at 1, 3, 7.
    add(&mut window, 1);
    add(&mut window, 3);
    add(&mut window, 7);

    // Event at ts = 11 closes (0,10] and should report [1,3,7].
    add(&mut window, 11);

    // Second window (10,20]: events at 13, 19.
    add(&mut window, 13);
    add(&mut window, 19);

    // Event at ts = 21 closes (10,20] and should report [11, 13, 19].
    add(&mut window, 21);

    let reports = reported.borrow();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0], vec![1, 3, 7]);
    assert_eq!(reports[1], vec![11, 13, 19]);
}