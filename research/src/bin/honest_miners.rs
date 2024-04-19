use mining_sim::prelude::*;

fn main() {
    let sim = SimulationBuilder::new()
        .add_miner(Honest::new())
        .add_miner(Honest::new())
        .rounds(10000)
        .power_dist(PowerDistribution::Equal)
        .build()
        .unwrap();

    let results_builder = sim.run_all().unwrap();

    let results = results_builder.revenue().build();

    println!("{}", results);
}
