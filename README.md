# Collatz Conjecture


This is a little side-project to get familiar with low-level thread-worker pools and some other high-performance aspects of Rust.

The goal of this implementation is to find circles in the `3n + 1` conjecture and thereby proof it wrong.
The algorithm works under the premise, that all numbers to `2^68` [have already been checked](https://en.wikipedia.org/wiki/Collatz_conjecture#Experimental_evidence), which saves us quite a bit of work.


## Implementation

At the start of the program, a worker pool with the size of `2 * logical threads` is started.

### Data Structures and Channels

**Task Dequeue**

This queue is used to distribute tasks between the worker pool.
Each worker is responsible to get the next task by itself, by simply popping the next element from the dequeue.

**Result Channel**

A single `mpsc` channel exists, which is used by the threads to send their results back to the main thread.

### Worker

The worker's job is to check, whether for each given number the main conjecture is valid.
I.e. whether we'll end in a `1;4;2` loop.

If that's the case, the checked number will be sent back via the result channel to inform the main thread that this number is checked.

### Main Thread

This thread is responsible for pushing new numbers into the worker queue and thereby keeping the workers busy.

Furthermore, finished numbers from workers are collected and each calculated number saved.
The largest natural number, that's connected to 0 without any gaps is saved as the next "safe" number and saved to disk.
That way, we can just continue where we left of last time.


## Run it

This project expects the latest stable Rust version.
It might be compatible with previous versions, but there's no guarantee it works.

Just execute:

```
cargo run
```
