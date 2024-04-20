# concurrent-ds
Lock-Free Data Structures
This repository contains the implementation of various lock-free data structures in Rust. These data structures are designed to perform efficiently in concurrent scenarios without traditional locking mechanisms.

#### Prerequisites
Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed on your machine.

#### Installing
To start using these data structures, clone the repository and build the project:

git clone [https://github.com/pandeyvibhu/lock-free-data-structures.git](https://github.com/pandeyvibhu/concurrent-ds.git)
cd concurrent-ds
cargo build

#### Running the benchmarks
To benchmark the performance of the lock-free data structures, you can use the integrated benchmarking tool provided by Cargo. Run the following command:
cargo bench

This command will execute all benchmarks defined in the project and output the performance metrics, which helps in evaluating the efficiency of the lock-free implementations.

#### Implemented Data Structures
Currently, the repository includes the following lock-free data structures:

Queue: Based on this research paper: [Simple, Fast, and Practical Non-Blocking and Blocking
Concurrent Queue Algorithms](https://www.cs.rochester.edu/~scott/papers/1996_PODC_queues.pdf)
