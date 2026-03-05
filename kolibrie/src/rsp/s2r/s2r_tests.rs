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
    use crate::rsp::s2r::Tick;
    use crate::rsp::s2r::window::WindowTriple;

    #[test]
    fn test_window() {
        let mut report = Report::new();
        report.add(ReportStrategy::OnWindowClose);

        let mut window =
            CSPARQLWindow::new(10, 2, report, Tick::TimeDriven, "test_window".to_string());

        // When windows are reported, the receiver will receive window contents
        let receiver = window.register_consumer();

        // The consumer will consume the received window content
        let consumer = Consumer::new();
        consumer.start(receiver);

        for time in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", time),
                p: "p".to_string(),
                o: "o".to_string(),
            };

            window.add_to_window(triple, time);
        }

        window.stop();
        thread::sleep(Duration::from_secs(1));
        assert_eq!(5, consumer.len());
    }


    #[test]
    fn test_window_with_callback() {
        let mut report = Report::new();
        report.add(ReportStrategy::OnWindowClose);

        let mut window: CSPARQLWindow<WindowTriple> =
            CSPARQLWindow::new(10, 2, report, Tick::TimeDriven, "test_window".to_string());

        let received_data = Arc::new(Mutex::new(Vec::new()));
        let received_data_for_callback = Arc::clone(&received_data);
        let call_back = move |content| {
            println!("Content: {:?}", content);
            received_data_for_callback.lock().unwrap().push(content);
        };
        window.register_callback(Box::new(call_back));

        for time in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", time),
                p: "p".to_string(),
                o: "o".to_string(),
            };
            window.add_to_window(triple, time);
        }

        window.stop();
        assert_eq!(5, received_data.lock().unwrap().len());
    }
}