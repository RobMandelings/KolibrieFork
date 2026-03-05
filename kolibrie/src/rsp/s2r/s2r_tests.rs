use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use log::debug;
use crate::rsp::s2r::window::WindowContent;

/// Part of the Consumer struct
#[allow(dead_code)]
struct ConsumerData<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    data: Mutex<Vec<WindowContent<I>>>,
}

#[allow(dead_code)]
struct Consumer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    inner: Arc<ConsumerData<I>>,
}

#[allow(dead_code)]
impl<I> Consumer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    fn new() -> Consumer<I> {
        Consumer {
            inner: Arc::new(ConsumerData {
                data: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Start listening for content sending in a different thread
    /// If content is received, push content to the consumer_temp (clone of inner consumer)
    fn start(&self, receiver: Receiver<WindowContent<I>>) {
        let consumer_temp = self.inner.clone();
        thread::spawn(move || loop {
            match receiver.recv() {
                // .revc() is a blocking operation (wait until you get result or err)
                Ok(content) => {
                    debug!("Found graph {:?}", content);
                    consumer_temp.data.lock().unwrap().push(content);
                }
                Err(_) => {
                    debug!("Shutting down!");
                    break;
                }
            }
        });
    }
    fn len(&self) -> usize {
        self.inner.data.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsp::s2r::reporting::{Report, ReportStrategy};
    use crate::rsp::s2r::sparql_window::CSPARQLWindow;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use crate::rsp::s2r::test_logging::init_logging;
    use crate::rsp::s2r::Tick;
    use crate::rsp::s2r::window::WindowTriple;

    fn add_triples_to_window(window: &mut CSPARQLWindow<WindowTriple>) {
        for time in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", time),
                p: "p".to_string(),
                o: "o".to_string(),
            };

            window.add_to_window(triple, time);
        }
    }

    type SharedVec<T> = Arc<Mutex<Vec<T>>>;

    fn setup_callback<T: Debug>() -> (SharedVec<T>, impl Fn(T)) {
        let received_data = Arc::new(Mutex::new(Vec::new()));

        let received_data_for_callback = Arc::clone(&received_data);
        // Function that will be called to consume the reported window content
        let callback_fn = move |content| {
            println!("Content: {:?}", content);
            received_data_for_callback.lock().unwrap().push(content);
        };

        (received_data, callback_fn)

    }

    #[test]
    fn test_window() {
        init_logging();

        let report = Report::with_strategies(vec![ReportStrategy::OnWindowClose]);
        let mut window =
            CSPARQLWindow::new(10, 2, report, Tick::TimeDriven, "test_window".to_string());

        // When windows are reported, the receiver will receive window contents
        let receiver = window.register_consumer();

        // The consumer will consume the received window content
        let consumer = Consumer::new();
        consumer.start(receiver);
        add_triples_to_window(&mut window);

        window.stop();
        thread::sleep(Duration::from_secs(1));
        assert_eq!(5, consumer.len());
    }


    #[test]
    fn test_window_with_callback() {
        let report = Report::with_strategies(vec![ReportStrategy::OnWindowClose]);
        let mut window: CSPARQLWindow<WindowTriple> =
            CSPARQLWindow::new(10, 2, report, Tick::TimeDriven, "test_window".to_string());

        let (received_data, callback_fn) = setup_callback();

        window.register_callback(Box::new(callback_fn));
        add_triples_to_window(&mut window);

        window.stop();
        assert_eq!(5, received_data.lock().unwrap().len());
    }
}