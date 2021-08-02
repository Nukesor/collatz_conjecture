use std::{cmp::Reverse, collections::BinaryHeap};

use color_eyre::eyre::Result;
use crossbeam::channel::Receiver;

use crate::DEFAULT_MAX_PROVEN_NUMBER;

#[allow(dead_code)]
pub fn min_heap(receiver: Receiver<u128>) -> Result<()> {
    // This heap is used to store all numbers that can't be reached from 1 via the sequence of
    // natural numbers yet. This happens, if we hit one especially hard number and the other
    // threads solve a lot of problems in the meantime.
    let mut backlog: BinaryHeap<Reverse<u128>> = BinaryHeap::new();

    // This is used to store the next highest natural number, that's connected to 1 via the
    // sequence of natural numbers.
    let mut highest_sequential_number = DEFAULT_MAX_PROVEN_NUMBER - 1;

    let mut counter = 0;
    loop {
        let number = receiver.recv()?;
        // Check if we got the next number in the sequence of natural numbers.
        if number == highest_sequential_number + 1 {
            highest_sequential_number = number;
            // Once we get the next number, check if we find the following numbers in our,
            // backlog as well.
            loop {
                // Check if the next number exists
                if let Some(number) = backlog.peek() {
                    // If the number checks out and is the next item in the sequence, we pop it
                    // and set it as the new highest number.
                    if number.0 == highest_sequential_number + 1 {
                        highest_sequential_number = backlog.pop().expect("We just peeked it").0;
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

        // We only print stuff every X iterations, as printing is super slow.
        // We also only update the highest_sequential_number during this interval.
        if counter == 5_000_000 {
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
