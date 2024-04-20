
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use myqueue::queue::{Queue, LockQueue, SingleVecLockQueue};
use std::thread::spawn;
use std::sync::{Arc, Barrier};

//Benchmarking the lockfree queue
fn bench_lockfree_queue(c: &mut Criterion) {
    let queue = Queue::new();
    c.bench_function("lockfree_queue", |b| {
        b.iter(|| {
            queue.enqueue(black_box(1000000));
            queue.dequeue();
        })
    });
}

//Benchmarking the lockfree concurrent queue
fn bench_lockfree_concurrent_queue(c: &mut Criterion) {
    let queue = Arc::new(Queue::<usize>::new());
    let barrier = Arc::new(Barrier::new(2));
    
    c.bench_function("lockfree_concurrent_queue", |b| {
        b.iter(|| {
            let barrier_clone = Arc::clone(&barrier);
            let queue_clone1 = Arc::clone(&queue);
            let queue_clone2 = Arc::clone(&queue);

            let handle = spawn(move || {
                barrier_clone.wait();
                queue_clone1.enqueue(black_box(1000000));
            });

            // The main thread also waits on the barrier to ensure synchronization
            barrier.wait();

            // Perform dequeue operation in the main test thread
            queue_clone2.dequeue();

            handle.join().unwrap();
        })
    });
}

//Benchmarking the lock queue with two lists
fn bench_lock_queue(c: &mut Criterion) {
    let queue = LockQueue::new();
    c.bench_function("lock_queue", |b| {
        b.iter(|| {
            queue.enqueue(black_box(1000000));
            queue.dequeue();
        })
    });
}

//Benchmarking the lock queue with two lists in a concurrent setting
fn bench_lock_concurrent_queue(c: &mut Criterion) {
    let queue = Arc::new(Queue::<usize>::new());
    let barrier = Arc::new(Barrier::new(2));
    
    c.bench_function("lock_concurrent_queue", |b| {
        b.iter(|| {
            let barrier_clone = Arc::clone(&barrier);
            let queue_clone1 = Arc::clone(&queue);
            let queue_clone2 = Arc::clone(&queue);

            let handle = spawn(move || {
                barrier_clone.wait();
                queue_clone1.enqueue(black_box(1000000));
            });

            // The main thread also waits on the barrier to ensure synchronization
            barrier.wait();

            // Perform dequeue operation in the main test thread
            queue_clone2.dequeue();

            handle.join().unwrap();
        })
    });
}

//Benchmarking the lock queue with a single list
fn bench_single_vec_lock_queue(c: &mut Criterion) {
    let queue = SingleVecLockQueue::new();
    c.bench_function("single_vec_lock_queue", |b| {
        b.iter(|| {
            queue.enqueue(black_box(1000000));
            queue.dequeue();
        })
    });
}

//Benchmarking the lock queue with a single list in a concurrent setting
fn bench_single_vec_lock_concurrent_queue(c: &mut Criterion) {
    let queue = Arc::new(SingleVecLockQueue::<usize>::new());
    let barrier = Arc::new(Barrier::new(2));
    
    c.bench_function("single_vec_lock_concurrent_queue", |b| {
        b.iter(|| {
            let barrier_clone = Arc::clone(&barrier);
            let queue_clone1 = Arc::clone(&queue);
            let queue_clone2 = Arc::clone(&queue);

            let handle = spawn(move || {
                barrier_clone.wait();
                queue_clone1.enqueue(black_box(1000000));
            });

            // The main thread also waits on the barrier to ensure synchronization
            barrier.wait();

            // Perform dequeue operation in the main test thread
            queue_clone2.dequeue();

            handle.join().unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_lockfree_queue,
    bench_lock_queue,
    bench_single_vec_lock_queue,
    bench_lockfree_concurrent_queue,
    bench_lock_concurrent_queue,
    bench_single_vec_lock_concurrent_queue
);
criterion_main!(benches);