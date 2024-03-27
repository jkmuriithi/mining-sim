use std::{error::Error, iter};

use mining_sim::prelude::*;

const GAMMA: f64 = 0.0;

fn main() -> Result<(), Box<dyn Error>> {
    // let start = Instant::now();

    let simulation = SimulationBuilder::new()
        .rounds(1000)
        .repeat_all(500)
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            MinerId::from(2),
            GAMMA,
        )))
        .add_miner(Selfish::new())
        .miner_power_iter(MinerId::from(2), iter::repeat(0.35).take(300))
        .build()?;

    let results_builder = simulation.run_all()?;

    let results = results_builder
        .average(Average::Mean)
        .all()
        .mining_power_func(
            2.into(),
            format!("Ideal SM Revenue (gamma={})", GAMMA),
            selfish_revenue(GAMMA),
        )
        .format(Format::CSV)
        .build();

    println!("{}", results);

    // println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());
    Ok(())
}
