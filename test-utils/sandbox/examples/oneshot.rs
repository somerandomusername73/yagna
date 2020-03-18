use futures::channel::oneshot;
use std::{thread, time::Duration};

fn main() {
    let (sender, receiver) = oneshot::channel::<i32>();

    thread::spawn(|| {
        println!("THREAD: sleeping zzz...");
        thread::sleep(Duration::from_millis(2000));
        println!("THREAD: i'm awake! sending.");
        sender.send(3).unwrap();
    });

    println!("MAIN: doing some useful stuff");

    futures::executor::block_on(async {
        println!("MAIN: waiting for msg...");
        println!("MAIN: got: {:?}", receiver.await)
    });
}
