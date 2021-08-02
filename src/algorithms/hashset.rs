use std::collections::HashSet;

use color_eyre::eyre::Result;
use crossbeam::channel::Receiver;

use crate::DEFAULT_MAX_PROVEN_NUMBER;

#[allow(dead_code)]
pub fn hashset(receiver: Receiver<u128>) -> Result<()> {
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
