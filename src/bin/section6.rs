use std::cell::UnsafeCell;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
struct ArcData<T> {
    ref_count: AtomicUsize,       // Arc count
    alloc_ref_count: AtomicUsize, // Arc count + Weak count
    data: UnsafeCell<Option<T>>,  // Weakポインタが残ってなかったらNone
}
struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}
unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}
impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
    fn upgrade(&self) -> Option<Arc<T>> {
        let mut n: usize = self.data().ref_count.load(Relaxed);
        loop {
            if n == 0 {
                return None;
            }
            assert!(n < usize::MAX);
            if let Err(e) = self
                .data()
                .ref_count
                .compare_exchange_weak(n, n + 1, Relaxed, Relaxed)
            {
                n = e;
                continue;
            }
            return Some(Arc { weak: self.clone() });
        }
    }
}
impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Weak { ptr: self.ptr }
    }
}
impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}
struct Arc<T> {
    weak: Weak<T>,
}
unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}
impl<T> Arc<T> {
    // メモリ領域確保にBoxを使用し、leakで排他的所有権を破棄
    // NonNull::fromでポインタに変換
    fn new(data: T) -> Arc<T> {
        Arc {
            weak: Weak {
                ptr: NonNull::from(Box::leak(Box::new(ArcData {
                    ref_count: AtomicUsize::new(1),
                    alloc_ref_count: AtomicUsize::new(1),
                    data: UnsafeCell::new(Some(data)),
                }))),
            },
        }
    }
    fn data(&self) -> &ArcData<T> {
        unsafe { self.weak.ptr.as_ref() }
    }
    fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(Relaxed) == 1 {
            fence(Acquire);
            let arc_data: &mut ArcData<T> = unsafe { arc.weak.ptr.as_mut() };
            let option: &mut Option<T> = arc_data.data.get_mut();
            let data: &mut T = option.as_mut().unwrap();
            Some(data)
        } else {
            None
        }
    }
    fn downgrade(arc: &Self) -> Weak<T> {
        arc.weak.clone()
    }
}
impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let ptr: *mut Option<T> = self.weak.data().data.get();
        unsafe { (*ptr).as_ref().unwrap() }
    }
}
impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak: Weak<T> = self.weak.clone();
        if weak.data().ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Arc { weak }
    }
}
impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            let ptr: *mut Option<T> = self.weak.data().data.get();
            unsafe { (*ptr) = None };
        }
    }
}
#[test]
fn test() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
    struct DetectDrop;
    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Relaxed);
        }
    }
    // let x: Arc<(&str, DetectDrop)> = Arc::new(("hello", DetectDrop));
    // let y: Arc<(&str, DetectDrop)> = x.clone();
    // let t: std::thread::JoinHandle<()> = std::thread::spawn(move || {
    //     assert_eq!(x.0, "hello");
    // });
    // assert_eq!(y.0, "hello");
    // t.join().unwrap();
    // drop(y);
    // assert_eq!(NUM_DROPS.load(Relaxed), 1);
    let x: Arc<(&str, DetectDrop)> = Arc::new(("hello", DetectDrop));
    let y: Weak<(&str, DetectDrop)> = Arc::downgrade(&x);
    let z: Weak<(&str, DetectDrop)> = Arc::downgrade(&x);
    let t: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let y: Arc<(&str, DetectDrop)> = y.upgrade().unwrap();
        assert_eq!(y.0, "hello");
    });
    assert_eq!(x.0, "hello");
    t.join().unwrap();
    assert_eq!(NUM_DROPS.load(Relaxed), 0);
    assert!(z.upgrade().is_some());
    drop(x);
    assert_eq!(NUM_DROPS.load(Relaxed), 1);
    assert!(z.upgrade().is_none());
}
fn main() {}
