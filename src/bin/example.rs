use std::{error::Error, time::Instant};

use mining_sim::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let simulation = SimulationBuilder::new()
        .rounds(100000)
        .repeat_all(20)
        .add_miner(Honest::new())
        .add_miner(NDeficit::new(1))
        .miner_power_iter(MinerId::from(2), (25..=50).percent())
        .build()?;

    let results_builder = simulation.run_all()?;

    let results = results_builder
        .average(Average::Median)
        .rounds()
        .revenue()
        .strategy_names()
        .mining_power_func(
            MinerId::from(2),
            "Ideal SM Revenue (Gamma=0.0)",
            selfish_revenue(0.0),
        )
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());

    Ok(())
}
