use std::time::Instant;

use mining_sim::miner::{ties::TieBreaker, Honest, Selfish};

fn main() {
    let start = Instant::now();

    let simulation = mining_sim::create()
        .rounds(1000000)
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            2.into(),
            0.4,
        )))
        .add_miner(Selfish::new())
        .with_miner_alphas(2, (0..50).step_by(2).map(|n| n as f64 / 100.0))
        // .with_miner_alphas(2, [0.44])
        .build()
        .unwrap();

    let results = simulation.run();
    let data = results.calculate_revenue().build_data();

    println!("{}", data);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());
}
