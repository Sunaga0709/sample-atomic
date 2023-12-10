// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread;
// use std::time::Duration;
// fn main() {
//     static STOP: AtomicBool = AtomicBool::new(false);
//     let background_thread: thread::JoinHandle<()> = thread::spawn(|| {
//         while !STOP.load(Relaxed) {
//             dbg!("some work");
//             thread::sleep(Duration::from_secs(2));
//         }
//     });
//     for line in std::io::stdin().lines() {
//         match line.unwrap().as_str() {
//             "help" => println!("command: help, stop"),
//             "stop" => {
//                 println!("input stop");
//                 break;
//             }
//             cmd => println!("unknown command: {cmd:?}"),
//         }
//     }
//     STOP.store(true, Relaxed);
//     background_thread.join().unwrap();
// }

// use std::sync::atomic::AtomicUsize;
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread;
// use std::time::Duration;
// fn main() {
//     let num_done: AtomicUsize = AtomicUsize::new(0);
//     thread::scope(|s: &thread::Scope<'_, '_>| {
//         s.spawn(|| {
//             for i in 0..10 {
//                 println!("long process");
//                 thread::sleep(Duration::from_secs(1));
//                 num_done.store(i + 1, Relaxed);
//             }
//         });
//         loop {
//             let n: usize = num_done.load(Relaxed);
//             if n == 10 {
//                 break;
//             }
//             println!("Working..{n}/10 done");
//             thread::sleep(Duration::from_secs(1));
//         }
//     });
//     println!("Done!");
// }

// use std::sync::atomic::AtomicUsize;
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread;
// use std::time::Duration;
// fn main() {
//     let num_done: AtomicUsize = AtomicUsize::new(0);
//     let main_thread: thread::Thread = thread::current();
//     thread::scope(|s: &thread::Scope<'_, '_>| {
//         s.spawn(|| {
//             for i in 0..10 {
//                 println!("long process");
//                 thread::sleep(Duration::from_secs(1));
//                 num_done.store(i+1, Relaxed);
//                 main_thread.unpark(); // start main thread
//             }
//         });
//         loop {
//             let n: usize = num_done.load(Relaxed);
//             if n == 10 {
//                 break;
//             }
//             println!("Working.. {n}/10 done");
//             thread::park_timeout(Duration::from_secs(1)); // wait 1sec
//         }
//     });
//     println!("\nDone");
// }

// use std::sync::atomic::AtomicU64;
// use std::sync::atomic::Ordering::Relaxed;
// fn get_x() -> u64 {
//     static X: AtomicU64 = AtomicU64::new(0);
//     let mut x: u64 = X.load(Relaxed);
//     if x == 0 {
//         x = calculate_x();
//         X.store(x, Relaxed)
//     }
//     x
// }

// use std::sync::atomic::AtomicI32;
// use std::sync::atomic::Ordering::Relaxed;
// fn main() {
//     let a: AtomicI32 = AtomicI32::new(100);
//     let b: i32 = a.fetch_add(23, Relaxed); // set 100 + 23, return 100
//     let c: i32 = a.load(Relaxed);
//     println!("b == 100: {}", b == 100_i32);
//     println!("c == 123: {}", c == 123);
// }

// use std::sync::atomic::AtomicUsize;
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread::{self, Scope};
// use std::time::Duration;
// fn main() {
//     let num_done: &AtomicUsize = &AtomicUsize::new(0); // ref for 4thread
//     thread::scope(|s: &Scope<'_, '_>| {
//         for _ in 0..4 {
//             s.spawn(move || {
//                 let mut current_sum: usize = 0;
//                 for _ in 0..25 {
//                     current_sum += 1;
//                     thread::sleep(Duration::from_millis(100));
//                     num_done.fetch_add(1, Relaxed);
//                 }
//                 println!(
//                     "thread: {:?}, result: {}",
//                     thread::current().id(),
//                     current_sum
//                 );
//             });
//         }
//         loop {
//             let n: usize = num_done.load(Relaxed);
//             if n == 100 {
//                 break;
//             }
//             println!("Working.. {n}/100 done");
//             thread::sleep(Duration::from_millis(100));
//         }
//     });
//     println!("\nDone");
// }

// use std::sync::atomic::{AtomicU64, AtomicUsize};
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread::{self, Scope};
// use std::time::{Duration, Instant};
// fn main() {
//     let num_done: &AtomicUsize = &AtomicUsize::new(0);
//     let total_time: &AtomicU64 = &AtomicU64::new(0);
//     let max_time: &AtomicU64 = &AtomicU64::new(0);
//     thread::scope(|s: &Scope<'_, '_>| {
//         for t in 0..4 {
//             s.spawn(move || {
//                 for i in 0..25 {
//                     let start: Instant = Instant::now();
//                     println!(" --> long process thread: ({t}), process: {i}");
//                     thread::sleep(Duration::from_secs(1));
//                     let time_taken: u64 = start.elapsed().as_micros() as u64;
//                     num_done.fetch_add(1, Relaxed);
//                     total_time.fetch_add(time_taken, Relaxed);
//                     max_time.fetch_max(time_taken, Relaxed);
//                 }
//             });
//         }
//         loop {
//             let total_time: Duration = Duration::from_micros(total_time.load(Relaxed));
//             let max_time: Duration = Duration::from_micros(max_time.load(Relaxed));
//             let n: usize = num_done.load(Relaxed);
//             if n == 100 {
//                 break;
//             }
//             if n == 0 {
//                 println!("Working.. nothing done yet.");
//             } else {
//                 println!("Working.. {n}/100 done, {:?} average, {max_time:?} peak", total_time/ n as u32);
//             }
//             thread::sleep(Duration::from_secs(1));
//         }
//     });
//     println!("\nDone");
// }

// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::{Duration, Instant};
// fn main() {
//     let num_done: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
//     let total_time: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
//     let max_time: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
//     let mut handles: Vec<thread::JoinHandle<()>> = vec![];
//     for t in 0..4 {
//         let num_done: Arc<Mutex<i32>> = Arc::clone(&num_done);
//         let total_time: Arc<Mutex<u64>> = Arc::clone(&total_time);
//         let max_time: Arc<Mutex<u64>> = Arc::clone(&max_time);
//         handles.push(thread::spawn(move || {
//             for i in 0..25 {
//                 let start: Instant = Instant::now();
//                 println!(" --> long process thread: ({t}), process: {i}");
//                 thread::sleep(Duration::from_secs(1));
//                 let time_taken: u64 = start.elapsed().as_micros() as u64;
//                 let mut num_done: std::sync::MutexGuard<'_, i32> = num_done.lock().unwrap();
//                 *num_done += 1;
//                 let mut total_time: std::sync::MutexGuard<'_, u64> = total_time.lock().unwrap();
//                 *total_time += time_taken;
//                 let mut max_time: std::sync::MutexGuard<'_, u64> = max_time.lock().unwrap();
//                 *max_time = (*max_time).max(time_taken);
//             }
//         }));
//     }
//     for handle in handles {
//         handle.join().unwrap();
//     }
//     let num_done: i32 = *num_done.lock().unwrap();
//     let total_time: u64 = *total_time.lock().unwrap();
//     let max_time: u64 = *max_time.lock().unwrap();
//     let average_time: Duration = if num_done > 0 {
//         Duration::from_micros(total_time / num_done as u64)
//     } else {
//         Duration::from_micros(0)
//     };
//     println!(
//         "{num_done}/100 done, {:?} average, {:?} peak",
//         average_time,
//         Duration::from_micros(max_time)
//     );
//     println!("\nDone");
// }

// use std::sync::atomic::AtomicU32;
// use std::sync::atomic::Ordering::Relaxed;
// // fn allocate_new_id() -> u32 { // panic
// //     static NEXT_ID: AtomicU32 = AtomicU32::new(1);
// //     let id: u32 = NEXT_ID.fetch_add(1, Relaxed);
// //     assert!(id < 1000, "too many ids");
// //     id
// // }
// fn allocate_new_id() -> u32 { // abort
//     static NEXT_ID: AtomicU32 = AtomicU32::new(1);
//     let id: u32 = NEXT_ID.fetch_add(1, Relaxed);
//     if id > 1000 {
//         NEXT_ID.fetch_sub(1, Relaxed);
//         panic!("too many ids");
//     }
//     id
// }
// fn main () {
//     for _ in 0..1001 {
//         println!("{}", allocate_new_id());
//     }
// }

// use std::atomic::AtomicU32;
// use std::atomic::Ordering::Relaxed;
// fn increment(a: &AtomicU32) {
//     let mut current: u32 = a.load(Relaxed);
//     loop {
//         let new: u32 = current + 1;
//         match a.compare_exchange(current, new, Relaxed, Relaxed) {
//             Ok(_) => return,
//             Err(v) => crrent = v,
//         }
//     }
// }

// use std::sync::atomic::AtomicU32;
// use std::sync::atomic::Ordering::Relaxed;
// fn allocate_new_id() -> u32 {
//     static NEXT_ID: AtomicU32 = AtomicU32::new(0);
//     let mut id: u32 = NEXT_ID.load(Relaxed);
//     loop {
//         assert!(id < 100, "too many ids");
//         match NEXT_ID.compare_exchange_weak(id, id + 1, Relaxed, Relaxed) {
//             Ok(_) => return id,
//             Err(v) => id = v,
//         }
//     }
// }
// fn main() {
//     for _ in 0..1001 {
//         let id: u32 = allocate_new_id();
//         println!("{id}");
//     }
// }

use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::{self, Scope};
fn get_key() -> (u32, bool) {
    static KEY: AtomicU32 = AtomicU32::new(0); // 未初期化
    let key: u32 = KEY.load(Relaxed);
    if key == 0 {
        let new_key: u32 = 1234_u32;
        match KEY.compare_exchange(key, new_key, Relaxed, Relaxed) {
            Ok(_) => (new_key, true), // 初期化に成功したら初期化した値を返す
            Err(v) => (v, false),     // 初期化に失敗（初期化済み）したら、既存の値を返す
        }
    } else {
        (key, false) // 初期化に失敗（初期化済み）したら、既存の値を返す
    }
}
fn main() {
    thread::scope(|s: &Scope<'_, '_>| {
        for _ in 0..10 {
            s.spawn(|| {
                let (key, is_inited): (u32, bool) = get_key();
                println!(
                    "thread: {:?}, key: {key}, is inited: {}",
                    thread::current().id(),
                    is_inited,
                );
            });
        }
    });
}
