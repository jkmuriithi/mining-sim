use std::{error::Error, time::Instant};

use mining_sim::{prelude::*, results::selfish_revenue};

const GAMMA: f64 = 0.5;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..50).map(|n| n as PowerValue / 100.0);
    let simulation = SimulationBuilder::new()
        .rounds(100000)
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            2.into(),
            GAMMA,
        )))
        .add_miner(Selfish::new())
        .miner_power_iter(2.into(), alpha)
        .build()?;

    let data = simulation.run_all()?;

    let results = data
        .all()
        .constant("Gamma", GAMMA)
        .mining_power_func(2.into(), "Ideal SM Revenue", selfish_revenue(GAMMA))
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());

    Ok(())
}
