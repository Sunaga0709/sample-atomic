use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
struct Lock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}
impl<T: Send> Lock<T> {
    fn new(val: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(val),
        }
    }
    fn lock(&self) -> LockGuard<T> {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop(); // thread::sleepなどとは違い他スレッドにリソースを譲らない
        }
        // lockメソッドを呼ばない限りLockGuardは取り出せない
        // LockGuardが生存していればロックされていることになる
        LockGuard { lock: self }
    }
    /// 安全性： lockが返した&mut Tの所有権が消費されてなければならない
    /// （Tに対する参照をどこかに残しておいてはならない）
    #[allow(unused)]
    fn unlock(&self) {
        self.locked.store(false, Release)
    }
}
unsafe impl<T> Sync for Lock<T> where T: Send {} // TではなくLockに対してSync（Tは同時アクセスされないようにしているため）
struct LockGuard<'a, T> {
    // 'aによりLockGuardがLockより長く生存しないことを明示する
    lock: &'a Lock<T>,
}
impl<T> Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // 安全性： この型が存在すること自体がロックを排他的に取得したことを保証する
        unsafe { &*self.lock.value.get() }
    }
}
impl<T> DerefMut for LockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // 安全性： この型が存在すること自体がロックを排他的に取得したことを保証する
        unsafe { &mut *self.lock.value.get() }
    }
}
unsafe impl<T> Send for LockGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for LockGuard<'_, T> where T: Sync {}
impl<T> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}
use std::thread::{self, Scope};
fn main() {
    let x: Lock<Vec<u32>> = Lock::new(Vec::new());
    thread::scope(|s: &Scope<'_, '_>| {
        s.spawn(|| x.lock().push(1));
        s.spawn(|| {
            let mut g: LockGuard<Vec<u32>> = x.lock();
            g.push(2);
            g.push(2);
        });
    });
    let g: LockGuard<Vec<u32>> = x.lock();
    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
}
