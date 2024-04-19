use anyhow::Result;
use mining_sim::prelude::*;

const GAMMA: f64 = 0.5;

fn main() -> Result<()> {
    let sim = SimulationBuilder::new()
        .add_miner(Honest::with_tie_breaker(TieBreaker::FavorMinerProb(
            MinerId::from(2),
            GAMMA,
        )))
        .add_miner(NDeficit::new(1))
        .miner_power_iter(MinerId::from(2), (0..=50).percent())
        .rounds(10000)
        .repeat_all(200)
        .build()?;

    let results = sim
        .run_all()?
        .average(Average::Mean)
        .all()
        .mining_power_func(
            MinerId::from(2),
            "Ideal Revenue",
            selfish_revenue(GAMMA),
        )
        .mining_power_func(MinerId::from(2), "Honest Revenue", |p| p)
        .format(Format::CSV)
        .build();

    println!("{}", results);

    Ok(())
}
