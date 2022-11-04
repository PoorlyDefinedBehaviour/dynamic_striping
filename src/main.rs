use std::{
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
  time::Duration,
};

struct Stripe {
  /// AtomicUsize is 8 bytes.
  value: AtomicUsize,
  /// A cache line is usually 64 bytes. Add padding to ensure that stripes do not share cache lines to avoid false sharing.
  /// cat /sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size
  _padding: [u8; 56],
}

struct Adder {
  stripes: Vec<Stripe>,
}

impl Adder {
  fn new(num_threads: usize) -> Self {
    let mut stripes = Vec::with_capacity(num_threads);
    stripes.resize_with(num_threads, || Stripe {
      value: AtomicUsize::new(0),
      _padding: [0; 56],
    });

    Self { stripes }
  }

  fn increment(&self, thread_id: usize) {
    self.stripes[thread_id].value.fetch_add(1, Ordering::SeqCst);
  }

  fn sum(&self) -> usize {
    let mut sum = 0;

    for stripe in self.stripes.iter() {
      sum += stripe.value.load(Ordering::SeqCst);
    }

    sum
  }
}

fn main() {
  let adder = Arc::new(Adder::new(4));

  let mut handles = vec![];
  for thread_id in 0..4 {
    let adder_clone = Arc::clone(&adder);

    handles.push(std::thread::spawn(move || {
      for _ in 0..3 {
        adder_clone.increment(thread_id);
        std::thread::sleep(Duration::from_secs(1));
      }
    }));
  }

  std::thread::spawn(move || {
    for _ in 0..5 {
      println!("sum={}", adder.sum());
      std::thread::sleep(Duration::from_secs(1));
    }
  })
  .join()
  .unwrap();

  for handle in handles.into_iter() {
    handle.join().unwrap();
  }
}
