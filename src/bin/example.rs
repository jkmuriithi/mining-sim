use std::{error::Error, time::Instant};

use mining_sim::{
    miner::{Honest, Selfish},
    simulation::results::OutputFormat,
};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..=50).step_by(2).map(|n| n as f64 / 100.0);
    let simulation = mining_sim::SimulationBuilder::new()
        .with_rounds(100000)
        .add_miner(Selfish::new())
        .add_miner(Honest::new())
        .with_miner_power_iter(1, alpha)
        .repeat_all(5)
        .build()?;

    let data = simulation.run_all()?;
    let results = data
        .averaged()
        .with_miner_names()
        .with_revenue()
        .with_rounds()
        .with_longest_chain_length()
        .with_format(OutputFormat::CSV)
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());

    Ok(())
}
