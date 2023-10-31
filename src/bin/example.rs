use std::time::Instant;

use mining_sim::miner::Honest;

fn main() {
    let start = Instant::now();

    let simulation = mining_sim::create()
        .average_of(5)
        .rounds(100000)
        .add_miner(Honest::new())
        .add_miner(Honest::new())
        .with_alphas([0.3, 0.7])
        .with_equal_alphas()
        .with_miner_alpha(1, 0.9)
        .build()
        .unwrap();

    let results = simulation.run();
    let data = results.calculate_revenue().build_data();

    println!("{}", data);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());
}
