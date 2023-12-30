use atomic_wait::{wait, wake_one};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut, Drop};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread::{self, Scope};
use std::time::{Duration, Instant};
struct Mutex<T> {
    state: AtomicU32, // 0: unlock, 1: lock（他に待機スレッドなし）, 2: lock（他に待機スレッドあり）
    value: UnsafeCell<T>,
}
impl<T> Mutex<T> {
    fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // unlock
            value: UnsafeCell::new(value),
        }
    }
    fn lock(&self) -> MutexGuard<T> {
        // ロックされていたら（1）分岐に流入
        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            lock_contended(&self.state);
        }
        MutexGuard { mutex: self }
    }
}
fn lock_contended(state: &AtomicU32) {
    let mut spin_count: i32 = 0;
    // 待機スレッドがない場合のみスピンロックする
    while state.load(Relaxed) == 1 && spin_count < 100 {
        spin_count += 1;
        std::hint::spin_loop();
    }
    // ロックされてなかったらstateを1に更新し（ロック）返却
    if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
        return;
    }
    // stateを2に更新し、すでにロックされていたらスレッドを停止する
    while state.swap(2, Acquire) != 0 {
        wait(state, 2);
    }
}
unsafe impl<T> Sync for Mutex<T> where T: Send {}
struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // 他に待機スレッドがあればそれを起動する
        if self.mutex.state.swap(0, Release) == 2 {
            wake_one(&self.mutex.state);
        }
    }
}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}
fn main() {
    let m: Mutex<u32> = Mutex::new(0_u32);
    std::hint::black_box(&m);
    let start: Instant = Instant::now();
    thread::scope(|s: &Scope| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration: Duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}
