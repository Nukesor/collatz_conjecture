# Collatz Conjecture


This is a little side-project to get familiar with low-level thread worker-pools and some other high-performance aspects of Rust.

The goal of this implementation is to find circles in the `3n + 1` conjecture and thereby proof it wrong.
The algorithm works under the premise, that all numbers to `2^68` [have already been checked](https://en.wikipedia.org/wiki/Collatz_conjecture#Experimental_evidence), which saves us quite a bit of work.

The current (very naive) implementation checks about 150 million numbers per second with 12 threads on a i7-8700k.

## Run it

This project expects the latest stable Rust version.
It might be compatible with previous versions, but there's no guarantee it works.

Just execute:

```
cargo run
```

# Implementation

## Data Structures and Channels

#### Atomic u128 counter

This counter is used to distribute tasks in the worker pool.
Each worker is responsible to get the next task by itself, by simply incrementing this counter.

#### Result Channel

A single `mpsc` channel exists, which is used by the threads to send their results back to the main thread.

## Worker

The worker's job is to check, whether for each given number the main conjecture is valid.
I.e. whether we'll end in a `1;4;2` loop.

If that's the case, the checked number will be sent back via the result channel to inform the main thread that this number is checked.

## Main Thread

This thread is responsible for pushing new numbers into the worker queue and thereby keeping the workers busy.

Furthermore, finished numbers from workers are collected and each calculated number is processed.
This is needed to determine the largest checked number, that can be reached from 0 in the sequence of natural numbers.
That way, we know how far we were able to compute results and know where to continue working.

Interestingly, storing these intermediate results turns out to be quite a problem.

A few variables:

- `HIGHEST_SEQUENTIAL_NUMBER` The last number in the sequence of natural numbers, that was proven to not be cyclic.
- `HIGHEST_NUMBER` The highest number that was was returned from any thread at the current time.
- `result backlog` The collection that contains all numbers that haven't been processed yet.
- `channel backlog` The amount of messages that accumulate in the mpsc channel, as the main thread cannot keep up.

All results below have been made with a threadcount of `logical cpus - 1`.
That way the main thread still got one processor exclusively.

#### MinHeap

The first iteration was using a simple binary MinHeap.
All elements that weren't `HIGHEST_SEQUENTIAL_NUMBER + 1` were stored on that heap. \
As soon as `HIGHEST_SEQUENTIAL_NUMBER  + 1` was solved, the heap was checked until no further subsequent number were found.

This approach turned out to be infeasible, as storing and extracting elements took way too long.
This was due to the nature of the internal binary tree and the amount of items that needed to be stored simultaneously. \
As some calculations took over a second, hundreds of thousands of elements had to be inserted into the heap in the meantime.
Once the long running calculation finished, they then had to be extracted again.

This caused not only the MinHeap to temporarily grow to high numbers (several million), but also the channel backlog to steadily increase.

#### HashSet

The second iteration took a different approach.
Instead of pushing all elements that weren't `HIGHEST_SEQUENTIAL_NUMBER + 1`, only elements that were missing between `HIGHEST_SEQUENTIAL_NUMBER` and `HIGHEST_NUMBER` were saved.
The maximum size of our result backlog with this approach equals the amount of spawned threads, as there can never be more missing numbers than running jobs.

Even though having this few slots, the underlying binary tree of the HashSet turned out to still be too slow.

Even though the max amount of items in the HashSet never exceeded the thread count, the messages in the channel still rapidly grew.
The main thread was just not capable of keeping up with the influx of numbers.

#### Vec

The third iteration took the same approach as before, but a simpler datastructure was utilized. \
As we know, that the amount of slots in our result backlog are rather manageable, a binary tree might just be overkill.

It turns out, that this works quite fine.

However, an interesting phenomenon could be observed.
Over the course of a few hours, the channel backlog sometimes rapidly increased (up to several hundred million entries), until it finally hit a point at which the main thread started to catch up.

#### Vec with fixed size

The forth iteration was a minor change from the previous approach. \
In the previous implementation, removing items took `O(n)` for searching an item in the backlog and further `O(n)` for removing the item.

I tried to be smart.
The idea was to never changing the size of the vector and to mark empty slots with `0`.

Adding values to the vector was always a `O(n)` linear scan worst-case scenario.
However it was also `O(n)` for multiple items, which is neat.

Removing items resulted in a linear scan `O(n)` with an `O(1)` deletion operation (insert `0`).
In theory, this should have been a bit faster. In practice, there wasn't much of a difference.


### Batch Sizes

Having the suspicion, that sending each message takes up a lot of the perfomance, I introduced batches for each worker.
And indeed, processing a few million numbers before reporting back turns out to be quite the performance boost.

This sped up the amount of numbers on 11 thread from about 5 million to 150 million.
Turns out, sending a lot of messages from many threads via an mpsc channel is quite expensive.

This makes all previous updates pretty useless, but they were a fun problem to have nevertheless :D.
