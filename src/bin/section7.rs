use std::hint::black_box;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Instant;
#[repr(align(64))]
struct Aligned(AtomicU64);
static A: [Aligned; 3] = [
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
];
fn main() {
    black_box(&A);
    thread::spawn(|| loop {
        A[0].0.store(0, Relaxed);
        A[2].0.store(0, Relaxed);
    });
    let start: Instant = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].0.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}
