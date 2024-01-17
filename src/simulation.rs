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
pub mod power_dist;
pub mod results;

pub use builder::{SimulationBuildError, SimulationBuilder};
pub use power_dist::{PowerDistribution, PowerDistributionError};
pub use results::SimulationResults;

/// Runs a simulation of the blockchain mining game, according to the given
/// parameters.
///
/// ## Details
/// [Miner::get_action] is called on each [Miner] instance based on their
/// insertion order (the order they were added to the corresponding
/// [SimulationBuilder]).
pub struct Simulation {
    rounds: usize,
    average_of: usize,
    miners: Vec<Box<dyn Miner>>,
    power_dists: Vec<Vec<f64>>,
    initial_blockchain: Option<Blockchain>,
}

impl Simulation {
    /// Runs the simulation with all configured power distributions.
    pub fn run_all(mut self) -> SimulationResults {
        let mut rng = rand::thread_rng();
        let mut chains = vec![];
        let mut miner_blocks = vec![];

        for i in 0..self.power_dists.len() {
            // Construct probability distribution from miner weights
            println!("{:?}", self.power_dists[i]);
            let dist = WeightedIndex::new(&self.power_dists[i]).unwrap();
            for _ in 0..self.average_of {
                let run = self.run_single(&dist, &mut rng);
                chains.push(run.0);
                miner_blocks.push(run.1);
            }
        }

        let Simulation {
            rounds,
            average_of,
            miners,
            power_dists,
            ..
        } = self;
        SimulationResults::new(
            rounds,
            average_of,
            chains,
            miners,
            power_dists,
            miner_blocks,
        )
    }

    /// Runs the configured simulation once and returns the resulting
    /// blockchain.
    fn run_single(
        &mut self,
        dist: &WeightedIndex<f64>,
        rng: &mut ThreadRng,
    ) -> (Blockchain, HashMap<MinerID, Vec<BlockID>>) {
        let mut chain = self.initial_blockchain.clone().unwrap_or_default();
        let mut miners = self.miners.clone();

        let mut miner_blocks: HashMap<MinerID, Vec<BlockID>> = HashMap::new();
        for r in 1..=self.rounds {
            let proposer = dist.sample(rng) + 1;

            for m in miners.iter_mut() {
                let miner = m.id();
                let block = if miner == proposer { Some(r) } else { None };

                match m.get_action(&chain, block) {
                    Action::Wait => (),
                    Action::Publish(block) => {
                        miner_blocks.entry(miner).or_default().push(block.id);
                        chain.publish(block).expect("valid block");
                    }
                    Action::PublishSet(blocks) => {
                        for block in blocks {
                            miner_blocks
                                .entry(miner)
                                .or_default()
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
