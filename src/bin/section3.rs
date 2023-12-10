// fn cal_func(a: &mut i32, b: &mut i32) {
//     // 各処理がそれぞれに依存していないので、コンパイラが順番を入れ替える可能性あり
//     // bがキャッシュされていればbから行う可能性もある
//     *a += 1;
//     *b += 1;
//     *a += 1;
// }
// fn main() {
//     let mut a: i32 = 1_i32;
//     let mut b: i32 = 1_i32;
//     cal_func(&mut a, &mut b);
//     println!("a: {a}, b: {b}");
// }

// use std::sync::atomic::AtomicU32;
// use std::sync::atomic::Ordering::Relaxed;
// use std::thread::{self, JoinHandle};
// static X: AtomicU32 = AtomicU32::new(0);
// fn f() {
//     let x: u32 = X.load(Relaxed);
//     dbg!(&x);
//     assert!(x == 1 || x == 2);
// }
// fn main() {
//     X.store(1, Relaxed);
//     let t: JoinHandle<()> = thread::spawn(f); // <- これで先行発生関係を作成
//     X.store(2, Relaxed); // 別スレッドを起動している最中に実行される
//     t.join().unwrap(); // <- これで先行発生関係を作成
//     X.store(3, Relaxed);
// }

// use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
// use std::thread::{self, ScopedJoinHandle, Scope};
// static X: AtomicU32 = AtomicU32::new(0);
// // fn a() {
// //     X.fetch_add(5, Relaxed);
// //     X.fetch_add(10, Relaxed);
// // }
// fn a1() {
//     X.fetch_add(5, Relaxed);
// }
// fn a2() {
//     X.fetch_add(10, Relaxed);
// }
// fn b() {
//     let a: u32 = X.load(Relaxed);
//     let b: u32 = X.load(Relaxed);
//     let c: u32 = X.load(Relaxed);
//     let d: u32 = X.load(Relaxed);
//     println!("{a}, {b}, {c}, {d}");
// }
// fn main() {
//     thread::scope(|s: &Scope<'_, '_>| {
//         let mut handles: Vec<ScopedJoinHandle<'_, ()>> = Vec::new();
//         handles.push(s.spawn(|| a1())); // 先に起動するスレッドの方が早く終わる
//         handles.push(s.spawn(|| a2())); // 先に起動するスレッドの方が早く終わる
//         handles.push(s.spawn(|| b())); // アトミック操作よりスレッド起動の方が時間がかかる
//         // a();
//         // b();
//         for h in handles {
//             h.join().unwrap();
//         }
//     });
// }

// use std::sync::atomic::{AtomicBool, AtomicU32};
// use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
// use std::thread;
// use std::time::Duration;
// static DATA: AtomicU32 = AtomicU32::new(0);
// static READY: AtomicBool = AtomicBool::new(false);
// fn main() {
//     thread::spawn(|| {
//         DATA.store(123, Relaxed);
//         READY.store(true, Release); // Acquireロード後の更新された値（DATA）は更新されていることが保証される
//     });
//     while !READY.load(Acquire) { // 更新後読み込む
//         println!("loop {}", DATA.load(Relaxed));
//         thread::sleep(Duration::from_millis(100));
//     }
//     println!("main {}", DATA.load(Relaxed));
// }

use std::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};
use std::thread;
use std::time::Duration;
static mut DATA: u32 = 0_u32;
static READY: AtomicBool = AtomicBool::new(false);
fn main() {
    thread::spawn(|| {
        unsafe { DATA = 123 }; // READYがセットされてないのでDATAへのアクセスはない
        READY.store(true, Release);
    });
    while !READY.load(Acquire) {
        thread::sleep(Duration::from_millis(100));
        println!("waiting...");
    }
    println!("{}", unsafe { DATA });
}
