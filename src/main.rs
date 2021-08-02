use std::sync::Arc;
use std::thread;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use color_eyre::eyre::Result;
use crossbeam::channel::{unbounded, Sender};

static DEFAULT_MAX_PROVEN_NUMBER: u128 = 2u128.pow(64);

/// We have to implement our own non-moving vector, since the backlog is by far the slowest part of
/// the main thread. Without some kind of special datastructure, we're quickly accumulating a lot
/// of messages in our mpsc channel.
///
/// This is value is simply a vector of zeros with the last bit flipped.
static EMPTY_SLOT: u128 = 0;

/// The amount of threads that should be started.
///
/// This is at the same time the amount of slots in the backlog.
/// In theory, we'll never need more backlog slots than there are threads.
static THREAD_COUNT: usize = 24;

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
    let mut backlog: Vec<u128> = vec![EMPTY_SLOT; THREAD_COUNT];

    let mut counter = 0;
    let mut highest_number = DEFAULT_MAX_PROVEN_NUMBER - 1;
    // The highest number that's connected in the sequence of natural numbers from `(0..number)`.
    let mut highest_sequential_number = DEFAULT_MAX_PROVEN_NUMBER - 1;

    loop {
        let next_number = receiver.recv()?;

        if next_number > highest_number {
            // Add all missing numbers that haven't been returned yet.
            let mut backlog_slot_iter = 0..THREAD_COUNT;
            for missing in highest_number + 1..next_number {
                // Scan the vector for free slots (slots with 0)
                // By using a stateful-vector, we only do a single scan for multiple missing
                // elements.
                while let Some(slot) = backlog_slot_iter.next() {
                    let value = backlog[slot];
                    if value == 0 {
                        backlog[slot] = missing;
                        break;
                    }
                }
            }

            // Set the new number as the highest number.
            highest_number = next_number;
        } else {
            // The number must be in the backlog.
            for i in 0..backlog.len() {
                if backlog[i] == next_number {
                    backlog[i] = 0;
                    break;
                }
            }
        }

        // We only print stuff every X iterations, as printing is super slow.
        // We also only update the highest_sequential_number during this interval.
        if counter == 5_000_000 {
            // Find the smallest number in our backlog.
            // That number minus 1 is our last succesfully calculated value.
            backlog.sort();
            for i in backlog.iter() {
                if i == &0 {
                    continue;
                }
                highest_sequential_number = i - 1;
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
