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

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::{self, Scope};
use std::time::Duration;
fn main() {
    let num_done: &AtomicUsize = &AtomicUsize::new(0); // ref for 4thread
    thread::scope(|s: &Scope<'_, '_>| {
        for _ in 0..4 {
            s.spawn(move || {
                let mut current_sum: usize = 0;
                for _ in 0..25 {
                    current_sum += 1;
                    thread::sleep(Duration::from_millis(100));
                    num_done.fetch_add(1, Relaxed);
                }
                println!(
                    "thread: {:?}, result: {}",
                    thread::current().id(),
                    current_sum
                );
            });
        }
        loop {
            let n: usize = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 done");
            thread::sleep(Duration::from_millis(100));
        }
    });
    println!("\nDone");
}
