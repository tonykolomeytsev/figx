use std::{
    hash::Hash,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct Batcher<V, B, R>
where
    V: Eq + Hash + Clone,
    B: Batched<V, R>,
{
    buffer: Arc<Mutex<Vec<V>>>,
    buffer_cond: Arc<Condvar>,
    timeout: Duration,
    max_batch_size: usize,
    batched_op: B,
    in_progress: Arc<AtomicBool>,
    result_cond: Arc<Condvar>,
    result: Arc<Mutex<Option<Arc<R>>>>,
}

pub trait Batched<V, R> {
    fn execute(&self, batch: Vec<V>) -> R;
}

impl<V, B, R> Batcher<V, B, R>
where
    V: Eq + Hash + Clone,
    B: Batched<V, R>,
{
    pub fn new(max_batch_size: usize, timeout: Duration, batched_op: B) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(max_batch_size))),
            buffer_cond: Arc::new(Condvar::new()),
            timeout,
            max_batch_size,
            batched_op,
            result: Arc::new(Mutex::new(None)),
            in_progress: Arc::new(AtomicBool::new(false)),
            result_cond: Arc::new(Condvar::new()),
        }
    }

    pub fn batch(&self, value: V) -> Arc<R> {
        let mut buffer = self.buffer.lock().unwrap();
        while buffer.len() >= self.max_batch_size {
            buffer = self.buffer_cond.wait(buffer).unwrap();
        }

        buffer.push(value);

        // Notify others that buffer changed (in case batcher is sleeping)
        self.buffer_cond.notify_all();
        drop(buffer); // release buffer lock early

        // Try to become the batch leader
        let is_batcher = self
            .in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();

        if is_batcher {
            // Clear any old result
            {
                let mut result = self.result.lock().unwrap();
                *result = None;
            }

            // Only sleep if buffer not full yet
            let sleep_needed = {
                let buffer = self.buffer.lock().unwrap();
                buffer.len() < self.max_batch_size
            };
            if sleep_needed {
                // Wait to collect more values
                std::thread::sleep(self.timeout);
            }

            let mut buffer = self.buffer.lock().unwrap();
            let mut values = Vec::new();
            values.append(&mut buffer);

            // Notify threads blocked on full buffer
            self.buffer_cond.notify_all();

            let response = self.batched_op.execute(values);
            let response = Arc::new(response);

            let mut result = self.result.lock().unwrap();
            *result = Some(response.clone());

            self.in_progress.store(false, Ordering::SeqCst);
            self.result_cond.notify_all(); // notify waiting threads

            response
        } else {
            // Wait for result to be available
            let mut result = self.result.lock().unwrap();
            while result.is_none() {
                result = self.result_cond.wait(result).unwrap();
            }
            result.clone().unwrap()
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use std::{
        collections::HashSet,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use super::*;

    macro_rules! hash_set {
        [$( $x:expr ),*] => {
            [ $( $x ),* ].into_iter().collect::<HashSet<_>>()
        };
    }

    struct TestBatchedOp(Arc<AtomicUsize>);
    impl Batched<i32, HashSet<i32>> for TestBatchedOp {
        fn execute(&self, batch: Vec<i32>) -> HashSet<i32> {
            self.0.fetch_add(1, Ordering::SeqCst);
            batch.into_iter().collect()
        }
    }

    #[test]
    fn run_batch_faster_than_timeout__EXPECT__all_requests_in_one_batch() {
        // Given
        let executions_count = Arc::new(AtomicUsize::new(0));
        let batcher = Arc::new(Batcher::new(
            10,
            Duration::from_millis(100),
            TestBatchedOp(executions_count.clone()),
        ));

        // When
        let handles = (0..5)
            .into_iter()
            .map(|i| {
                let batcher = Arc::clone(&batcher);
                std::thread::spawn(move || batcher.batch(i))
            })
            .collect::<Vec<_>>();
        let results = handles
            .into_iter()
            .map(|it| it.join().unwrap())
            .collect::<Vec<_>>();

        // Then
        assert_eq!(1, executions_count.load(Ordering::SeqCst));
        assert_eq!(5, results.len());
        let first_value = results.first().unwrap();
        assert!(results.iter().all(|it| it == first_value));
        assert_eq!(&hash_set![0, 1, 2, 3, 4], first_value.as_ref());
    }

    #[test]
    fn run_batch_slower_than_timeout__EXPECT__all_requests_in_one_two_batches() {
        // Given
        let executions_count = Arc::new(AtomicUsize::new(0));
        let batcher = Arc::new(Batcher::new(
            10,
            Duration::from_millis(500),
            TestBatchedOp(executions_count.clone()),
        ));

        // When
        let handles = (0..5)
            .into_iter()
            .map(|i| {
                let batcher = Arc::clone(&batcher);
                std::thread::spawn(move || {
                    if i > 1 {
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                    batcher.batch(i)
                })
            })
            .collect::<Vec<_>>();
        let results = handles
            .into_iter()
            .map(|it| it.join().unwrap())
            .collect::<Vec<_>>();

        // Then
        assert_eq!(2, executions_count.load(Ordering::SeqCst));
        assert_eq!(5, results.len());
        assert_eq!(&hash_set!(0, 1), results.iter().nth(0).unwrap().as_ref());
        assert_eq!(&hash_set!(0, 1), results.iter().nth(1).unwrap().as_ref());
        assert_eq!(&hash_set!(2, 3, 4), results.iter().nth(2).unwrap().as_ref());
        assert_eq!(&hash_set!(2, 3, 4), results.iter().nth(3).unwrap().as_ref());
        assert_eq!(&hash_set!(2, 3, 4), results.iter().nth(4).unwrap().as_ref());
    }

    #[test]
    fn run_batch_faster_than_timeout__buffer_overflow__EXPECT__all_requests_in_one_two_batches() {
        // Given
        let executions_count = Arc::new(AtomicUsize::new(0));
        let batcher = Arc::new(Batcher::new(
            3,
            Duration::from_millis(500),
            TestBatchedOp(executions_count.clone()),
        ));

        // When
        let handles = (0i32..5)
            .into_iter()
            .map(|i| {
                let batcher = Arc::clone(&batcher);
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(i as u64 * 10));
                    batcher.batch(i)
                })
            })
            .collect::<Vec<_>>();
        let results = handles
            .into_iter()
            .map(|it| it.join().unwrap())
            .collect::<Vec<_>>();

        // Then
        assert_eq!(2, executions_count.load(Ordering::SeqCst));
        assert_eq!(5, results.len());
        assert_eq!(&hash_set!(0, 1, 2), results.iter().nth(0).unwrap().as_ref());
        assert_eq!(&hash_set!(0, 1, 2), results.iter().nth(1).unwrap().as_ref());
        assert_eq!(&hash_set!(0, 1, 2), results.iter().nth(2).unwrap().as_ref());
        assert_eq!(&hash_set!(3, 4), results.iter().nth(3).unwrap().as_ref());
        assert_eq!(&hash_set!(3, 4), results.iter().nth(4).unwrap().as_ref());
    }
}
