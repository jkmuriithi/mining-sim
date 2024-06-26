use anyhow::Result;
use mining_sim::prelude::*;

fn main() -> Result<()> {
    // let start = Instant::now();

    let simulation = SimulationBuilder::new()
        .rounds(10000)
        .repeat_all(1000)
        .add_miner(Honest::new())
        .add_miner(Selfish::new())
        .miner_power(MinerId::from(2), 0.35)
        .build()?;

    let results_builder = simulation.run_all()?;

    let results = results_builder
        // .average(Average::Mean)
        .all()
        .mining_power_func(2.into(), "Ideal Revenue", selfish_revenue(0.0))
        .format(Format::CSV)
        .build();

    println!("{}", results);

    // println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());
    Ok(())
}
