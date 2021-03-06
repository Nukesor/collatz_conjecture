use std::sync::Arc;
use std::thread;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use color_eyre::eyre::Result;
use crossbeam::channel::{unbounded, Sender};
use num_format::{Locale, ToFormattedString};

mod algorithms;

static DEFAULT_MAX_PROVEN_NUMBER: u128 = 2u128.pow(64);

/// The amount of threads that should be started.
///
/// This is at the same time the amount of slots in the backlog.
/// In theory, we'll never need more backlog slots than there are threads.
static THREAD_COUNT: usize = 12;

/// The amount of numbers that will be processed by a thread in a single unit of work.
/// This drastically reduces the amount of messages that need to be send via the mpsc channel.
/// It also reduces the work the main thread has to do.
static BATCH_SIZE: u128 = 10_000_000;

/// At which interval of calculated numbers a new messages will be printed.
static REPORTING_SIZE: u128 = 1_000_000_000;

fn main() -> Result<()> {
    println!(
        "Starting at number: {}",
        DEFAULT_MAX_PROVEN_NUMBER.to_formatted_string(&Locale::en)
    );
    println!(
        "Batch size: {}",
        BATCH_SIZE.to_formatted_string(&Locale::en)
    );

    // A thread-safe atomic counter.
    // This counter is shared between all threads and used to get the next tasks.
    let counter: Arc<Atomic<u128>> = Arc::new(Atomic::new(DEFAULT_MAX_PROVEN_NUMBER));

    let (sender, receiver) = unbounded();

    // Spawn the worker pool
    spawn_threads(counter, sender)?;

    //algorithms::min_heap(receiver)
    //algorithms::hashset(receiver)
    //algorithms::vector(receiver)
    algorithms::fixed_vector(receiver)
}

/// Spin up twice as many threads as there are logical cores.
fn spawn_threads(counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
    for thread_id in 0..THREAD_COUNT {
        let counter_clone = Arc::clone(&counter);
        let sender_clone = sender.clone();
        thread::spawn(move || {
            if let Err(error) = thread_logic(thread_id, counter_clone, sender_clone) {
                eprintln!("Got error in thread:\n{:?}", error);
            };
        });
    }

    Ok(())
}

/// The main logic of the thread.
/// Check for circles and print a message if one is found.
///
/// Each thread processes a batch of numbers before reporting back.
fn thread_logic(_thread: usize, counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
    loop {
        let next_number = counter.fetch_add(BATCH_SIZE, Relaxed);
        for i in 0..BATCH_SIZE {
            let found_circle = find_circle(next_number + i);

            if found_circle {
                println!("Found circle for {}", next_number);
            }
        }
        sender.send(next_number)?;
    }
}

/// A very trivial implementation of the collatz conjecture algorithm.
#[inline(always)]
fn find_circle(mut number: u128) -> bool {
    let original = number;
    while number > DEFAULT_MAX_PROVEN_NUMBER {
        if number % 2 == 1 {
            number = number / 2;
        } else {
            number = (3 * number + 1) / 2
        }

        if number == original {
            return true;
        }
    }

    false
}
