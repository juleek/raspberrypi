use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

fn f(tx: std::sync::mpsc::Sender<i32>) {
   let mut a = 0;
   loop {
      let val = 2;
      tx.send(val).unwrap();
      thread::sleep(Duration::from_secs(1));
      a += 1;
      if a > 3 {
         break;
      }
   }
}

fn main() {
   let (tx, rx) = mpsc::channel();
   thread::spawn(move || f(tx));
   loop {
      let received = rx.recv();
      println!("{:?}", received);
   }
}


