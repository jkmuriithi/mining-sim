use std::time::Instant;

use mining_sim::miner::Honest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let first_miner_power = (0..50).step_by(2).map(|n| n as f64 / 100.0);
    let simulation = mining_sim::SimulationBuilder::new()
        .add_miner(Honest::new())
        .add_miner(Honest::new())
        .with_rounds(100000)
        .with_miner_power_iter(1, first_miner_power)
        .repeat_each(5)
        .build();

    let simulation = match simulation {
        Ok(sim) => sim,
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(e));
        }
    };

    simulation.run_all()?;
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());

    Ok(())
}
