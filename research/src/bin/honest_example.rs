use anyhow::Result;
use mining_sim::prelude::*;

fn main() -> Result<()> {
    let sim = SimulationBuilder::new()
        .add_miner(Honest::new())
        .add_miner(Honest::new())
        .miner_power_iter(MinerId::from(2), (0..=50).percent())
        .rounds(10000)
        // .repeat_all(200)
        .build()?;

    let results = sim
        .run_all()?
        // .average(Average::Mean)
        .all()
        .mining_power_func(MinerId::from(2), "Ideal Revenue", |p| p)
        .format(Format::CSV)
        .build();

    println!("{}", results);

    Ok(())
}
