use std::fmt::Debug;
use std::hash::Hash;
use crate::rsp::s2r::{WindowBounds, WindowContent};

/// Reporting Strategies define the conditions under which the engine emits the content of the window.
#[derive(Clone, Debug)]
pub enum ReportStrategy {
    NonEmptyContent,
    OnContentChange,
    OnWindowClose,
    Periodic(usize),
}
impl Default for ReportStrategy {
    fn default() -> Self {
        ReportStrategy::OnWindowClose
    }
}

pub struct Report<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    strategies: Vec<ReportStrategy>,
    last_change: WindowContent<I>,
}

impl<I> Report<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    pub fn new() -> Report<I> {
        Report {
            strategies: Vec::new(), // Reporting strategies to consider when checking whether window should be reported
            last_change: WindowContent::new(), // Used for the OnContentChange reporting strategy
        }
    }

    /// Adds a new reporting strategy to the report.
    pub fn add(&mut self, strategy: ReportStrategy) {
        self.strategies.push(strategy);
    }

    /// Returns true if the window should be reported.
    /// This only happens when all reporting strategies within the Vec<ReportStrategy>
    /// say that reporting strategy should be 'true'
    pub fn should_report_window(
        &mut self,
        window: &WindowBounds,
        content: &WindowContent<I>,
        ts: usize,
    ) -> bool {
        self.strategies.iter().all(|strategy| match strategy {
            ReportStrategy::NonEmptyContent => content.len() > 0,
            ReportStrategy::OnContentChange => {
                let comp = content.eq(&self.last_change);
                self.last_change = content.clone();
                comp
            }
            ReportStrategy::OnWindowClose => window.close < ts,
            ReportStrategy::Periodic(period) => ts % period == 0,
        })
    }
}