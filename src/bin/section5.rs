// use std::collections::VecDeque;
// use std::sync::{Condvar, Mutex};
// struct Channel<T> {
//     queue: Mutex<VecDeque<T>>,
//     item_ready: Condvar,
// }
// impl<T> Channel<T> {
//     fn new() -> Self {
//         Self {queue: Mutex::new(VecDeque::new()), item_ready: Condvar::new()}
//     }
//     fn send(&self, message: T) {
//         // Mutexロックを取得し、queue末尾に引数を追加する
//         self.queue.lock().unwrap().push_back(message);
//         // 条件変数で受信側に通知する
//         self.item_ready.notify_one();
//     }
//     fn receive(&self) -> T {
//         // Mutexロックを取得
//         let mut b: Vec<T> = self.queue.lock().unwrap();
//         // queue先頭から値が取れるまでループ、取得できたらそれを返す
//         loop {
//             if let Some(message) = b.pop_front {
//                 return message;
//             }
//             // queueに値がなかったら通知が来るまで待機する
//             // Condvar::waitは待機中アンロックしている
//             b = self.item_ready.wait(b).unwrap();
//         }
//     }
// }

// use std::cell::UnsafeCell;
// use std::mem::MaybeUninit;
// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::{Acquire, Relaxed};
// use std::sync::Arc;
// struct Channel<T> {
//     message: UnsafeCell<MaybeUninit<T>>,
//     ready: AtomicBool,
//     in_use: AtomicBool,
// }
// unsafe impl<T> Sync for Channel<T> where T: Send {}
// impl<T> Channel<T> {
//     const fn new() -> Self {
//         Self {
//             message: UnsafeCell::new(MaybeUninit::uninit()), // 未初期化
//             ready: AtomicBool::new(false),
//             in_use: AtomicBool::new(false),
//         }
//     }
//     unsafe fn send(&self, message: T) {
//         if self.in_use.swap(true, Relaxed) {
//             panic!("can not send more than one message!");
//         }
//         (*self.message.get()).write(message);
//         self.ready.store(true, Relaxed);
//     }
//     fn receive(&self) -> T {
//         if !self.ready.swap(false, Acquire) {
//             panic!("no message available!");
//         }
//         unsafe { (*self.message.get()).assume_init_read() }
//     }
//     fn is_ready(&self) -> bool {
//         self.ready.load(Acquire)
//     }
// }
// impl<T> Drop for Channel<T> {
//     fn drop(&mut self) {
//         if *self.ready.get_mut() {
//             unsafe {self.message.get_mut().assume_init_drop() }
//         }
//     }
// }
// fn main() {
//     use std::thread::{self, Scope, Thread};
//     let channel: Channel<&str> = Channel::new();
//     let t: Thread = thread::current();
//     thread::scope(|s: &Scope| {
//         s.spawn(|| {
//             t.unpark();
//             unsafe {channel.send("hello, world"); }
//         });
//         while !channel.is_ready() {
//             thread::park();
//         }
//         assert_eq!(channel.receive(), "hello, world");
//     });
// }

// use std::cell::UnsafeCell;
// use std::mem::MaybeUninit;
// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
// use std::sync::Arc;
// use std::thread::{self, Scope, Thread};
// struct Channel<T> {
//     message: UnsafeCell<MaybeUninit<T>>,
//     ready: AtomicBool,
// }
// // Arcは最大2つ（Sender、Receiver）
// struct Sender<T> {
//     channel: Arc<Channel<T>>,
// }
// struct Receiver<T> {
//     channel: Arc<Channel<T>>,
// }
// unsafe impl<T> Sync for Channel<T> where T: Send {}
// fn channel<T>() -> (Sender<T>, Receiver<T>) {
//     let a: Arc<Channel<T>> = Arc::new(Channel {
//         message: UnsafeCell::new(MaybeUninit::uninit()),
//         ready: AtomicBool::new(false),
//     });
//     (Sender { channel: a.clone() }, Receiver { channel: a })
// }
// impl<T> Sender<T> {
//     fn send(self, message: T) {
//         unsafe { (*self.channel.message.get()).write(message) };
//         self.channel.ready.store(true, Release);
//     }
// }
// impl<T> Receiver<T> {
//     fn is_ready(&self) -> bool {
//         self.channel.ready.load(Relaxed)
//     }
//     fn receive(self) -> T {
//         if !self.channel.ready.swap(true, Acquire) {
//             panic!("no message available!");
//         }
//         unsafe { (*self.channel.message.get()).assume_init_read() }
//     }
// }
// impl<T> Drop for Channel<T> {
//     fn drop(&mut self) {
//         if *self.ready.get_mut() {
//             unsafe { self.message.get_mut().assume_init_drop() }
//         }
//     }
// }
// fn main() {
//     thread::scope(|s: &Scope<'_, '_>| {
//         let (sender, receiver) = channel();
//         let t: Thread = thread::current();
//         s.spawn(move || {
//             sender.send("hello world");
//             t.unpark(); // senderで送信したら停止している別スレッドのループを抜けさせ、起動する
//         });
//         while !receiver.is_ready() {
//             thread::park(); // receiverで取得できるようになるまでスレッドを停止
//         }
//         assert_eq!(receiver.receive(), "hello world");
//     });
// }

use core::marker::PhantomData;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread::{self, Scope, Thread};
struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}
impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }
    // 引数にライフタイムを指定することでSender、Receiverの生存期間を指定
    // 今回の場合はChannelのメソッドとしてあるためライフタイムを省略可能
    // 可変参照（排他的）にすることで1ChannelについてSender、Receiverのセットが1組であることを保証する
    // Sender、Receiverがなくなれば排他借用が無効になり、再度このメソッドを呼び出すことができる
    fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        // 空で上書きし過去の状態を引き継がないようにする
        // ↑これにより過去のChannelに対してDropと同じ役割になる
        *self = Self::new();
        (
            Sender {
                channel: self,
                receiving_thread: thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}
unsafe impl<T> Sync for Channel<T> where T: Send {}
struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread, // 受信するスレッド
}
impl<T> Sender<'_, T> {
    fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
        self.receiving_thread.unpark();
    }
}
struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const ()>, // 生ポインタでSendは実装していない
}
impl<T> Receiver<'_, T> {
    fn is_ready(&self) -> bool {
        self.channel.ready.load(Acquire)
    }
    fn receive(self) -> T {
        if !(self.channel.ready.swap(false, Acquire)) {
            thread::park();
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}
fn main() {
    // // ブロック外で定義することにより、channelがsenderやreceiverより長く生存することを示す
    // let mut channel = Channel::new();
    // thread::scope(|s: &Scope<'_, '_>| {
    //     let (sender, receiver) = channel.split();
    //     let t: Thread = thread::current();
    //     s.spawn(move || {
    //         sender.send("hello world");
    //         t.unpark();
    //     });
    //     while !receiver.is_ready() {
    //         thread::park();
    //     }
    //     assert_eq!(receiver.receive(), "hello world", "unmatch channel value");
    // });
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("hello world");
        });
        assert_eq!(receiver.receive(), "hello world");
    });
}
