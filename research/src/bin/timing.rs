use std::time::Instant;

use anyhow::Result;
use mining_sim::prelude::*;

fn main() -> Result<()> {
    let start = Instant::now();

    let simulation = SimulationBuilder::new()
        .rounds(1_000_000)
        .repeat_all(125)
        .add_miner(Honest::new())
        .add_miner(Selfish::new())
        .miner_power(MinerId::from(2), 0.35)
        .build()?;

    let results = simulation
        .run_all()?
        .average(Average::Mean)
        .all()
        .mining_power_func(2.into(), "Ideal Revenue", selfish_revenue(0.0))
        .build();

    println!("{}", results);

    println!("elapsed time: {:.4} secs", start.elapsed().as_secs_f64());
    Ok(())
}
