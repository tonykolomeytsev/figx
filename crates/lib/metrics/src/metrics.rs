use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::Instant,
};

// region: COUNT

#[derive(Default)]
pub struct Counter(AtomicUsize);

impl Counter {
    pub fn set(&self, value: usize) {
        self.0.store(value, Ordering::SeqCst);
    }

    pub fn increment(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

// endregion: COUNT

// region: DURATION

#[derive(Default)]
pub struct Duration(AtomicU64);
pub struct DurationRecorder<'a> {
    parent: &'a Duration,
    start: Instant,
}

impl Duration {
    pub fn record(&self) -> DurationRecorder<'_> {
        DurationRecorder {
            parent: self,
            start: Instant::now(),
        }
    }

    pub fn get(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.0.load(Ordering::SeqCst))
    }
}

impl<'a> Drop for DurationRecorder<'a> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.parent
            .0
            .store(elapsed.as_millis() as u64, Ordering::SeqCst);
    }
}

// endregion: DURATION
