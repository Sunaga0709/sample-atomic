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

// use std::sync::atomic::{
//     AtomicBool,
//     Ordering::{Acquire, Release},
// };
// use std::thread;
// use std::time::Duration;
// static mut DATA: u32 = 0_u32;
// static READY: AtomicBool = AtomicBool::new(false);
// fn main() {
//     thread::spawn(|| {
//         unsafe { DATA = 123 }; // READYがセットされてないのでDATAへのアクセスはない
//         READY.store(true, Release);
//     });
//     while !READY.load(Acquire) {
//         thread::sleep(Duration::from_millis(100));
//         println!("waiting...");
//     }
//     println!("{}", unsafe { DATA });
// }

// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
// use std::thread::{self, Scope, ScopedJoinHandle};
// static mut DATA: String = String::new();
// static LOCKED: AtomicBool = AtomicBool::new(false);
// fn f() {
//     if LOCKED
//         // 成功時、読み取りを重視しているのでAcquireを指定
//         // falseにするReleaseストアを可視化
//         .compare_exchange(false, true, Acquire, Relaxed)
//         .is_ok()
//     {
//         // lockされてなかったら値を更新し、lockを解除する
//         unsafe { DATA.push('!') };
//         // 他から読み取られる前に更新することを保証する
//         LOCKED.store(false, Release);
//     }
// }
// fn main() {
//     thread::scope(|s: &Scope<'_, '_>| {
//         let mut handle: Vec<ScopedJoinHandle<'_, ()>> = Vec::new();
//         for _ in 0..10 {
//             handle.push(s.spawn(f));
//         }
//         for h in handle {
//             h.join().unwrap();
//         }
//     });
//     println!("{:?}", unsafe { DATA.clone() });
// }

// use std::sync::atomic::AtomicPtr;
// use std::sync::atomic::Ordering::{Acquire, Release};
// fn get_data() -> &'static Data {
//     static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());
//     let mut p = PTR.load(Acquire);
//     if p.is_null() { // nullだったら生成し置き換え可能なら置き換える
//         p = Box::into_raw(Box::new(generate_data()));
//         if let Err(e) = PTR.compare_exchage(
//             std::ptr::null_mut(), p, Release, Acquire,
//         ) {
//             // 置き換えに失敗したら解放し、セットされているものを「p」にセットする
//             drop(unsafe {Box::from_raw(p)});
//             p = e;
//         }
//     }
//     unsafe {&*p} // nullではなく、すでにセットされている値か新たに生成した値が入る
// }

// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::SeqCst;
// use std::thread::{self, JoinHandle};
// static A: AtomicBool = AtomicBool::new(false);
// static B: AtomicBool = AtomicBool::new(false);
// static mut S: String = String::new();
// fn main() {
//     let a: JoinHandle<_> = thread::spawn(|| {
//         dbg!("called thread A");
//         A.store(true, SeqCst);
//         // Bスレッドが先に起動していたらここの分岐に入らない（SeqCstで保証されている）
//         if !B.load(SeqCst) {
//             unsafe { S.push('!') };
//         }
//         println!("thread A, S value{:?}", unsafe { &S });
//     });

//     let b: JoinHandle<_> = thread::spawn(|| {
//         dbg!("called thread B");
//         B.store(true, SeqCst);
//         // Aスレッドが先に起動していたらここの分岐に入らない（SeqCstで保証されている）
//         if !A.load(SeqCst) {
//             unsafe { S.push('!') };
//         }
//         println!("thread B, S value{:?}", unsafe { &S });
//     });

//     a.join().unwrap();
//     b.join().unwrap();
//     println!("result{:?}", unsafe { &S });
// }

use std::sync::atomic::{
    fence, AtomicBool,
    Ordering::{Acquire, Relaxed, Release},
};
use std::thread;
use std::time::Duration;
static mut DATA: [u64; 10] = [0; 10];
const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];
fn some_calculation(u: u64) -> u64 {
    u * 10
}
fn main() {
    // ループで複数スレッドを起動し、計算処理結果を該当インデックスの値を書き出す
    // 書き出すが完了したらREADYの該当インデックスのフラグを更新
    for i in 0..10 {
        thread::spawn(move || {
            thread::sleep(Duration::from_micros(20));
            let data: u64 = some_calculation(i);
            unsafe { DATA[i as usize] = data };
            // 該当インデックスが書き出し済みでメインスレッドから読み込める状態であることを示す
            READY[i as usize].store(true, Release);
        });
    }
    // thread::sleep(Duration::from_nanos(1));
    // アトミック変数から各真偽値を取り出し配列にし直す
    let ready: [bool; 10] = std::array::from_fn(|i: usize| READY[i].load(Relaxed));
    // 完了しているものがあれば結果を出力する
    if ready.contains(&true) {
        fence(Acquire); // この時点でReleaseストアされたものは保証する
        for i in 0..10 {
            if ready[i] {
                println!("data{i} = {}", unsafe { DATA[i] });
            }
        }
    }
}
