// use std::thread;
// fn main() {
//     // let numbers: Vec<u32> = vec![1, 2, 3];
//     let x: [i32; 3] = [1, 2, 3];
//     thread::scope(|s| {
//         s.spawn(|| {
//             println!("length: {}", x.len());
//         });
//         s.spawn(|| {
//             for n in x {
//                 println!("{n}");
//             }
//         });
//     });
// }

// use std::rc::Rc;
// fn main() {
//     let a: Rc<[u32; 3]> = Rc::new([1, 2, 3]);
//     let b: Rc<[u32; 3]> = a.clone();
//     assert_eq!(a.as_ptr(), b.as_ptr());
//     println!("{:?}", a.as_ptr());
//     println!("{:?}", b.as_ptr());
//     println!("a: {:?}", a);
//     println!("b: {:?}", b);
// }

// use std::thread;
// use std::rc::Rc;
// fn main() {
//     let a: Rc<u32> = Rc::new(123_u32);
//     thread::spawn(move || { // error
//         dbg!(a);
//     });
// }

// use std::sync::{Mutex, MutexGuard};
// use std::thread;
// use std::time::Duration;
// fn main() {
//     let n: Mutex<u32> = Mutex::new(0);
//     thread::scope(|s| {
//         for _ in 0..10 {
//             s.spawn(|| {
//                 println!("wait to unlock");
//                 let mut guard: MutexGuard<u32> = n.lock().unwrap();
//                 for _ in 0..100 {
//                     *guard += 1;
//                 }
//                 drop(guard);
//                 // println!("{guard:?}");
//                 thread::sleep(Duration::from_secs(1));
//             });
//         }
//     });
//     println!("result = {}", n.into_inner().unwrap());
// }

// use std::collections::VecDeque;
// use std::sync::Mutex;
// use std::thread;
// use std::time::Duration;
// fn main() {
//     let quere: Mutex<VecDeque<u32>> = Mutex::new(VecDeque::new());
//     thread::scope(|s| {
//         let t: thread::ScopedJoinHandle<'_, _> = s.spawn(|| loop {
//             let item: Option<u32> = quere.lock().unwrap().pop_front();
//             if let Some(item) = item {
//                 dbg!(item);
//             } else {
//                 thread::park();
//             }
//         });
//         for i in 0.. {
//             quere.lock().unwrap().push_back(i);
//             t.thread().unpark();
//             thread::sleep(Duration::from_secs(1));
//         }
//     });
// }

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
fn main() {
    let queue: Mutex<VecDeque<i32>> = Mutex::new(VecDeque::new());
    let not_empty: Condvar = Condvar::new();
    thread::scope(|s: &thread::Scope<'_, '_>| {
        s.spawn(|| loop {
            let mut q: MutexGuard<'_, VecDeque<i32>> = queue.lock().unwrap();
            let item: i32 = loop {
                if let Some(item) = q.pop_front() {
                    break item;
                } else {
                    q = not_empty.wait(q).unwrap();
                }
            };
            drop(q);
            dbg!(item);
        });
        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
