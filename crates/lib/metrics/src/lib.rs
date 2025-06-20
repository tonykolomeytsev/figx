mod metrics;
use dashmap::DashMap;
pub use metrics::*;
mod prom;
use std::{ops::Deref, sync::Arc};

#[derive(Default, Clone)]
pub struct Metrics(Arc<MetricsCollector>);

impl Deref for Metrics {
    type Target = MetricsCollector;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct MetricsCollector {
    durations: DashMap<&'static str, Arc<Duration>>,
    counters: DashMap<&'static str, Arc<Counter>>,
}

impl MetricsCollector {
    pub fn duration(&self, name: &'static str) -> Arc<Duration> {
        self.durations.entry(name).or_default().value().clone()
    }

    pub fn counter(&self, name: &'static str) -> Arc<Counter> {
        self.counters.entry(name).or_default().value().clone()
    }
}
