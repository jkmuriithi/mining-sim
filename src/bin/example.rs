use std::{error::Error, time::Instant};

use mining_sim::{
    miner::{selfish::selfish_revenue, Honest, NDeficit},
    PowerValue, SimulationBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let alpha = (0..50).map(|n| n as PowerValue / 100.0);
    let simulation = SimulationBuilder::new()
        .rounds(10000)
        .repeat_all(10)
        .add_miner(Honest::new())
        .add_miner(NDeficit::new(1))
        .miner_power_iter(2, alpha)
        .build()?;

    let data = simulation.run_all()?;

    let results = data
        .all()
        .averaged()
        .mining_power_func(2, "Ideal SM Revenue", selfish_revenue(0.0))
        .output_format(mining_sim::OutputFormat::CSV)
        .build();

    println!("{}", results);
    println!("Elapsed time: {:.4} secs", start.elapsed().as_secs_f64());

    Ok(())
}
