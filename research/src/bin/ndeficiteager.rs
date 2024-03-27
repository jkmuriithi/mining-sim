// Investigate the implementation of the N-Deficit Eager strategy.

use anyhow::Result;
use mining_sim::prelude::*;

const GAMMA: f64 = 0.00;

fn main() -> Result<()> {
    let sim = SimulationBuilder::new()
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            MinerId::from(2),
            GAMMA,
        )))
        .add_miner(NDeficitEager::new(1))
        .rounds(100000)
        .miner_power(MinerId::from(2), 0.40)
        .repeat_all(20)
        .build()?;

    let results = sim
        .run_all()?
        .strategy_names()
        .revenue()
        .mining_power_func(
            MinerId::from(2),
            "Ideal Selfish Miner Revenue",
            selfish_revenue(GAMMA),
        )
        .average(Average::Mean)
        .build();

    println!("{}", results);

    Ok(())
}
