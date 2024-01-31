use std::{error::Error, time::Instant};

use mining_sim::{
    miner::{Honest, NDeficit},
    tie_breaker::TieBreaker,
    OutputFormat, PowerValue, SimulationBuilder,
};

const GAMMA: f64 = 0.0;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..50).map(|n| n as PowerValue / 100.0);
    let simulation = SimulationBuilder::new()
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            2, GAMMA,
        )))
        .add_miner(NDeficit::new(2))
        .with_rounds(10000)
        .with_miner_power_iter(2, alpha)
        .repeat_all(5)
        .build()?;

    let data = simulation.run_all()?;

    let results = data
        .averaged()
        .with_revenue()
        .with_strategy_names()
        .with_format(OutputFormat::CSV)
        .with_mining_power_func(2, "Ideal NSM Revenue", nsm_rev)
        .with_mining_power_func(2, "Ideal SM Revenue", selfish_rev)
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());

    Ok(())
}

/// Ideal selfish miner revenue function from Eyal/Sirer
fn selfish_rev(a: PowerValue) -> f64 {
    (a * (1.0 - a).powi(2) * (4.0 * a + GAMMA * (1.0 - 2.0 * a)) - a.powi(3))
        / (1.0 - a * (1.0 + a * (2.0 - a)))
}

/// Ideal NSM miner reveue function from Weinberg/Ferrera
fn nsm_rev(a: PowerValue) -> f64 {
    (4.0 * a.powi(2) - 8.0 * a.powi(3) - a.powi(4) + 7.0 * a.powi(5)
        - 3.0 * a.powi(6))
        / (1.0 - a - 2.0 * a.powi(2) + 3.0 * a.powi(4) - 3.0 * a.powi(5)
            + a.powi(6))
}
