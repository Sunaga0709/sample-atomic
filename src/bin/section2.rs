use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;
fn main() {
    static STOP: AtomicBool = AtomicBool::new(false);
    let background_thread: thread::JoinHandle<()> = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            dbg!("some work");
            thread::sleep(Duration::from_secs(2));
        }
    });
    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("command: help, stop"),
            "stop" => {
                println!("input stop");
                break;
            }
            cmd => println!("unknown command: {cmd:?}"),
        }
    }
    STOP.store(true, Relaxed);
    background_thread.join().unwrap();
}
