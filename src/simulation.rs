use std::collections::HashMap;

use rand::{
    distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng,
};

use crate::{
    block::BlockID,
    blockchain::Blockchain,
    miner::{Action, Miner, MinerID},
};

pub mod builder;
pub mod results;

pub use builder::{create, SimulationBuilder};
pub use results::SimulationResults;

/// Runs a simulation of the blockchain mining game, according to the given
/// parameters.
pub struct Simulation {
    /// Number of data points to collect. Must be at least 1.
    average_of: usize,
    /// Number of rounds the simulation will last for.
    rounds: u64,
    /// All miners taking part in the simulation, sorted by
    /// [MinerID](crate::miner::MinerID). [Miner::get_action] will be called on
    /// each miner in order by ID.
    miners: Vec<Miner>,
    /// Parallel array containing the mining power of each miner in
    /// [Simulation::miners].
    miner_alphas: Vec<Vec<f64>>,
    /// Initial blockchain state used in the simulation.
    chain: Blockchain,
}

impl Simulation {
    /// Runs the configured simnulation and returns results.
    ///
    /// ## Details
    /// [Miner::get_action] is called on each [Miner] instance based on their
    /// insertion order (the order they were added to the corresponding
    /// [SimulationBuilder]).
    pub fn run(mut self) -> SimulationResults {
        let mut rng = rand::thread_rng();
        let mut chains = vec![];
        let mut miner_blocks = vec![];

        for i in 0..self.miner_alphas.len() {
            // Construct probability distribution from miner weights
            let dist = WeightedIndex::new(&self.miner_alphas[i]).unwrap();
            for _ in 0..self.average_of {
                let run = self.run_single(&dist, &mut rng);
                chains.push(run.0);
                miner_blocks.push(run.1);
            }
        }

        SimulationResults::new(
            self.average_of,
            self.miners,
            self.miner_alphas,
            miner_blocks,
            chains,
        )
    }

    /// Runs the configured simulation once and returns the resulting
    /// blockchain.
    fn run_single(
        &mut self,
        dist: &WeightedIndex<f64>,
        rng: &mut ThreadRng,
    ) -> (Blockchain, HashMap<MinerID, Vec<BlockID>>) {
        // Clone initial state
        let mut chain = self.chain.clone();
        let mut miners = self.miners.clone();

        let mut miner_blocks = HashMap::new();
        for r in 1..=self.rounds {
            let proposer = ((dist.sample(rng) + 1) as u64).into();

            for m in miners.iter_mut() {
                let miner = m.id();
                let block = if miner == proposer {
                    Some(r.into())
                } else {
                    None
                };

                match m.get_action(&chain, block) {
                    Action::Wait => (),
                    Action::Publish(block) => {
                        miner_blocks
                            .entry(miner)
                            .or_insert(vec![])
                            .push(block.id);
                        chain.publish(block).expect("valid block");
                    }
                    Action::PublishSet(blocks) => {
                        for block in blocks {
                            miner_blocks
                                .entry(miner)
                                .or_insert(vec![])
                                .push(block.id);
                            chain.publish(block).expect("valid block");
                        }
                    }
                }
            }
        }

        (chain, miner_blocks)
    }
}
