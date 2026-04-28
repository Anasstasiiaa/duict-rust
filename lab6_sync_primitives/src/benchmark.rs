use std::time::Instant;

fn main() {
    let start = Instant::now();

    // workload
    for _ in 0..1_000_000 {
        let _x = 1 + 2;
    }

    println!("Time: {:?}", start.elapsed());
}
