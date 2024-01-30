use std::{error::Error, time::Instant};

use mining_sim::{
    miner::{Honest, NDeficit, Selfish},
    PowerValue, SimulationBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..=50).step_by(2).map(|n| n as PowerValue / 100.0);
    let simulation = SimulationBuilder::new()
        .add_miner(Honest::new())
        .add_miner(NDeficit::new(1))
        .with_rounds(100000)
        .with_miner_power_iter(2, alpha)
        .repeat_all(5)
        .build()?;

    let data = simulation.run_all()?;
    let results = data
        .averaged()
        .with_rounds()
        .with_strategy_names()
        .with_revenue()
        .with_longest_chain_length()
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());

    Ok(())
}
