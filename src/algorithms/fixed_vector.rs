use color_eyre::eyre::Result;
use crossbeam::channel::Receiver;

use crate::{DEFAULT_MAX_PROVEN_NUMBER, THREAD_COUNT};

/// We have to implement our own non-moving vector, since the backlog is by far the slowest part of
/// the main thread. Without some kind of special datastructure, we're quickly accumulating a lot
/// of messages in our mpsc channel.
///
/// This is value is simply a vector of zeros with the last bit flipped.
static EMPTY_SLOT: u128 = 0;

#[allow(dead_code)]
pub fn fixed_vector(receiver: Receiver<u128>) -> Result<()> {
    // This is used to store all numbers that haven't been solved yet.
    // -> For instance, if the task for 10 completes, but 7, 8 and 9 haven't yet, these will be
    //      added to the backlog.
    //  In theory, there should never be more than `threadpool_count` elements in the backlog.
    let mut backlog: Vec<u128> = vec![EMPTY_SLOT; THREAD_COUNT];

    let mut highest_number = DEFAULT_MAX_PROVEN_NUMBER - 1;
    // The highest number that's connected in the sequence of natural numbers from `(0..number)`.
    let mut highest_sequential_number = DEFAULT_MAX_PROVEN_NUMBER - 1;

    let mut counter = 0;
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
