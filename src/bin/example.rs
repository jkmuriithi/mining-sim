use std::time::Instant;

use mining_sim::miner::{Honest, Selfish};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // let selfish_miner_power = (0..50).step_by(2).map(|n| n as f64 / 100.0);
    let simulation = mining_sim::SimulationBuilder::new()
        .add_miner(Honest::new())
        .add_miner(Selfish::new())
        .with_rounds(100000)
        .taking_average_of(5)
        .with_power_dist([0.1, 0.9])
        .build();

    let simulation = match simulation {
        Ok(sim) => sim,
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(e));
        }
    };
    let results = simulation.run_all();
    let data = results.calculate_revenue().build_data();

    println!("{}", data);
    println!("Elapsed time: {:.4}", start.elapsed().as_secs_f64());

    Ok(())
}
