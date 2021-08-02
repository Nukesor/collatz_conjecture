use std::cmp::Reverse;
use std::thread;
use std::{collections::BinaryHeap, sync::Arc};

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

    // This heap is used to store all numbers that can't be reached from 1 via the sequence of
    // natural numbers yet. This happens, if we hit one especially hard number and the other
    // threads solve a lot of problems in the meantime.
    let mut backlog: BinaryHeap<Reverse<u128>> = BinaryHeap::new();

    // This is used to store the next highest natural number, that's connected to 1 via the
    // sequence of natural numbers.
    let mut next_highest_number = DEFAULT_MAX_PROVEN_NUMBER;
    loop {
        while let Ok(number) = receiver.recv() {
            // Check if we got the next number in the sequence of natural numbers.
            if number == next_highest_number + 1 {
                next_highest_number = number;
                println!("Checked number {:?}", next_highest_number);
                // Once we get the next number, check if we find the following numbers in our,
                // backlog as well.
                loop {
                    // Check if the next number exists
                    if let Some(number) = backlog.peek() {
                        // If the number checks out and is the next item in the sequence, we pop it
                        // and set it as the new highest number.
                        if number.0 == next_highest_number + 1 {
                            next_highest_number = backlog.pop().expect("We just peeked it").0;
                            println!("Checked number {:?}", next_highest_number);
                        } else {
                            // If it isn't, we just break and wait.
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else {
                backlog.push(Reverse(number));
            }
        }
    }
}

/// Spin up twice as many threads as there are logical cores.
fn spawn_threads(counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
    let cpus = num_cpus::get();
    for _ in 0..cpus * 2 {
        let counter_clone = Arc::clone(&counter);
        let sender_clone = sender.clone();
        thread::spawn(|| {
            if let Err(error) = thread_logic(counter_clone, sender_clone) {
                eprintln!("Got error in thread:\n{:?}", error);
            };
        });
    }

    Ok(())
}

/// The main logic of the thread.
/// Check for circles and print a message if one is found.
fn thread_logic(counter: Arc<Atomic<u128>>, sender: Sender<u128>) -> Result<()> {
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
