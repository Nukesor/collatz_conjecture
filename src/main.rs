use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use color_eyre::eyre::Result;
use crossbeam::channel::{unbounded, Sender};

static DEFAULT_MAX_PROVEN_NUMBER: u128 = 2u128.pow(64);

fn main() -> Result<()> {
    // A thread-safe atomic counter.
    // This counter is shared between all threads and used to get the next tasks.
    let counter: Arc<Atomic<u128>> = Arc::new(Atomic::new(DEFAULT_MAX_PROVEN_NUMBER));

    let (sender, receiver) = unbounded();

    // Spawn the worker pool
    spawn_threads(counter, sender)?;

    // This heap is used to store all numbers that haven't been solved yet.
    // -> For instance, if the task for 10 completes, but 7, 8 and 9 haven't yet, these will be
    //      added to the backlog.
    //  In theory, there should never be more than `threadpool_count` elements in the backlog.
    let mut backlog: HashSet<u128> = HashSet::new();

    let mut counter = 0;
    let mut highest_number = DEFAULT_MAX_PROVEN_NUMBER - 1;
    // The highest number that's connected in the sequence of natural numbers from `(0..number)`.
    let mut highest_sequential_number = DEFAULT_MAX_PROVEN_NUMBER - 1;

    loop {
        let number = receiver.recv()?;

        if number > highest_number {
            // Add all missing numbers that haven't been returned yet.
            for i in highest_number + 1..number {
                backlog.insert(i);
            }

            // Set the new number as the highest number.
            highest_number = number;
        } else {
            // The number should be in the backlog.
            if !backlog.remove(&number) {
                panic!("Got smaller number that isn't in backlog: {}", number);
            };
        }

        // We only print stuff every X iterations, as printing is super slow.
        // We also only update the highest_sequential_number during this interval.
        if counter == 5_000_000 {
            // If there's still a backlog, the highest sequential number must be the smallest
            // number in the backlog -1
            if let Some(number) = backlog.iter().next() {
                highest_sequential_number = number - 1;
            } else {
                highest_sequential_number = highest_number;
            }

            println!(
                "max_number: {}, Channel size: {}, backlog size: {}",
                highest_sequential_number,
                receiver.len(),
                backlog.len()
            );

            // Reset the counter
            counter = 0;
        }

        counter += 1;
    }
}

/// Spin up twice as many threads as there are logical cores.
fn spawn_threads(counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
    let cpus = num_cpus::get();
    for thread_id in 0..cpus {
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
fn thread_logic(_thread: usize, counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
    loop {
        let next_number = counter.fetch_add(1, Relaxed);
        let found_circle = find_circle(next_number);

        if found_circle {
            println!("Found circle for {}", next_number);
        } else {
            sender.send(next_number)?;
        }
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
