Lab 5

Topic: Parallel execution of CPU-heavy tasks in Rust
Goal: Learn multithreading and task parallelization

Task Description
The previous image-processing application was improved by moving CPU-bound operations (decode, resize, encode, save) into parallel execution using the Rayon crate to distribute work across multiple threads.

Parallel Processing (example)

use rayon::prelude::\*;

entries.par_iter().enumerate().for_each(|(idx, entry)| {
process_entry(entry, &config, idx);
});

Execution Time Measurement (example)

use std::time::Instant;

let start = Instant::now();
// ... run processing ...
println!("Time: {:?}", start.elapsed());

Results

Before (single-thread): Time: 5.67s
After (rayon parallel): Time: 2.47s

Result

Performance improvement: ~56% faster execution

Conclusion

Parallel processing significantly improves performance for CPU-heavy image operations by utilizing multiple CPU cores more efficiently.
