use std::{
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, bounded};
use log::debug;

pub struct Batcher<V, B, R>
where
    V: Eq + Hash + Clone + Send + 'static,
    R: Send + Sync + 'static,
    B: Batched<V, R> + Send + Sync + 'static,
{
    tx: Sender<(V, Sender<Arc<R>>)>,
    _marker: PhantomData<B>,
}

pub trait Batched<V, R> {
    fn execute(&self, batch: Vec<V>) -> R;
}

impl<V, B, R> Batcher<V, B, R>
where
    V: Eq + Hash + Clone + Send + 'static,
    R: Send + Sync + 'static,
    B: Batched<V, R> + Send + Sync + 'static,
{
    pub fn new(max_batch_size: usize, timeout: Duration, batched_op: B) -> Self {
        let (tx, rx) = bounded::<(V, Sender<Arc<R>>)>(1024);
        let op = Arc::new(batched_op);
        thread::spawn(move || batch_loop(rx, max_batch_size, timeout, op));
        Self {
            tx,
            _marker: Default::default(),
        }
    }

    pub fn batch(&self, value: V) -> Arc<R> {
        let (resp_tx, resp_rx) = bounded(1);
        self.tx
            .send((value, resp_tx))
            .expect("Batcher thread crashed");
        resp_rx.recv().expect("Batcher thread crashed")
    }
}

fn batch_loop<V, R, B>(
    rx: Receiver<(V, Sender<Arc<R>>)>,
    max_batch_size: usize,
    timeout: Duration,
    batched_op: Arc<B>,
) where
    V: Eq + Hash + Clone + Send + 'static,
    R: Send + Sync + 'static,
    B: Batched<V, R> + Send + Sync + 'static,
{
    let mut buffer: Vec<(V, Sender<Arc<R>>)> = Vec::with_capacity(max_batch_size);

    loop {
        let start = Instant::now();
        match rx.recv() {
            Ok(first) => {
                buffer.push(first);

                // Try to fill up the rest of the batch or until timeout
                while buffer.len() < max_batch_size {
                    let remaining = timeout
                        .checked_sub(start.elapsed())
                        .unwrap_or(Duration::ZERO);
                    match rx.recv_timeout(remaining) {
                        Ok(item) => buffer.push(item),
                        Err(_) => break, // timeout
                    }
                }

                let values: Vec<V> = buffer.iter().map(|(v, _)| v.clone()).collect();
                debug!(target: "Batcher", "Executing batched operation...");
                let result = Arc::new(batched_op.execute(values));
                for (_, tx) in buffer.drain(..) {
                    let _ = tx.send(result.clone());
                }
            }
            Err(_) => break, // Channel closed
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
    #[ignore = "flaky on CI"]
    fn run_batch_faster_than_timeout__EXPECT__all_requests_in_one_batch() {
        // Given
        let executions_count = Arc::new(AtomicUsize::new(0));
        let batcher = Arc::new(Batcher::new(
            10,
            Duration::from_millis(100),
            TestBatchedOp(executions_count.clone()),
        ));

        // When
        let handles = (0..10)
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
        assert_eq!(10, results.len());
        let first_value = results.first().unwrap();
        assert!(results.iter().all(|it| it == first_value));
        assert_eq!(
            &hash_set![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            first_value.as_ref()
        );
    }

    #[test]
    #[ignore = "flaky on CI"]
    fn run_batch_slower_than_timeout__EXPECT__all_requests_in_one_two_batches() {
        // Given
        let executions_count = Arc::new(AtomicUsize::new(0));
        let batcher = Arc::new(Batcher::new(
            10,
            Duration::from_millis(150),
            TestBatchedOp(executions_count.clone()),
        ));

        // When
        let handles = (0..5)
            .into_iter()
            .map(|i| {
                let batcher = Arc::clone(&batcher);
                std::thread::spawn(move || {
                    if i > 1 {
                        std::thread::sleep(Duration::from_millis(200));
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
    #[ignore = "flaky on CI"]
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
