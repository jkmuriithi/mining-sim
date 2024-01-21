use std::{error::Error, time::Instant};

use mining_sim::{
    miner::{Honest, Selfish},
    SimulationBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..50).map(|n| n as f64 / 100.0);
    let simulation = SimulationBuilder::new()
        .with_rounds(1000000)
        .add_miner(Selfish::new())
        .add_miner(Honest::new())
        .with_miner_power_iter(1, alpha)
        .build()?;

    let data = simulation.run_all()?;
    let results = data
        .averaged()
        .with_strategy_names()
        .with_revenue()
        .with_longest_chain_length()
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());

    Ok(())
}
