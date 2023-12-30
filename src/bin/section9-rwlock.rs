use atomic_wait::{wait, wake_all, wake_one};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut, Drop};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
struct RwLock<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}
unsafe impl<T> Sync for RwLock<T> where T: Sync + Send {}
impl<T> RwLock<T> {
    const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // unlock
            value: UnsafeCell::new(value),
        }
    }
    fn read(&self) -> ReadGuard<T> {
        let mut s: u32 = self.state.load(Relaxed);
        loop {
            if s < u32::MAX {
                assert!(s != u32::MAX - 1, "too many readers");
                match self.state.compare_exchange_weak(s, s + 1, Acquire, Relaxed) {
                    Ok(_) => return ReadGuard { rwlock: self },
                    Err(e) => s = e,
                }
            }
            if s == u32::MAX {
                wait(&self.state, u32::MAX);
                s = self.state.load(Relaxed);
            }
        }
    }
    fn write(&self) -> WriteGuard<T> {
        while let Err(s) = self.state.compare_exchange(0, u32::MAX, Acquire, Relaxed) {
            wait(&self.state, s); // ロックされていたら待機
        }
        WriteGuard { rwlock: self }
    }
}
struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}
impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get() }
    }
}
impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.rwlock.state.fetch_sub(1, Release) == 1 {
            // 待機中のライタが存在すればそれを起動する
            wake_one(&self.rwlock.state);
        }
    }
}
struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}
impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get() }
    }
}
impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rwlock.value.get() }
    }
}
impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.state.store(0, Release);
        wake_all(&self.rwlock.state); // 待機中のリーダ、ライタを起動する
    }
}
fn main() {}
