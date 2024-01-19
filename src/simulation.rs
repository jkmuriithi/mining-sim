use std::collections::HashMap;

use rand::{
    distributions::{WeightedError, WeightedIndex},
    prelude::Distribution,
};

use crate::{
    blockchain::{BlockPublishingError, Blockchain},
    miner::{Action, Miner, MinerID},
};

pub mod builder;
pub mod power_dist;
pub mod results;

pub use builder::{SimulationBuildError, SimulationBuilder};
pub use power_dist::{PowerDistribution, PowerDistributionError};

use results::SimulationOutput;

/// Container for a group of simulations, which can be
#[derive(Debug, Clone)]
pub struct SimulationGroup {
    sims: Vec<Simulation>,
    repeat_all: usize,
}

impl SimulationGroup {
    pub fn new(repeat_all: usize) -> Self {
        assert_ne!(repeat_all, 0, "repeat_all must be greater than 0");
        Self {
            repeat_all,
            ..Default::default()
        }
    }

    pub fn add(&mut self, sim: Simulation) {
        self.sims.push(sim);
    }

    pub fn run_all(self) -> Result<Vec<SimulationOutput>, SimulationError> {
        let mut outputs = Vec::with_capacity(self.sims.len());
        for sim in self.sims {
            for _ in 0..(self.repeat_all - 1) {
                let sim_clone = sim.clone();
                outputs.push(sim_clone.run()?);
            }
            outputs.push(sim.run()?);
        }

        Ok(outputs)
    }
}

impl Default for SimulationGroup {
    fn default() -> Self {
        Self {
            repeat_all: 1,
            sims: vec![],
        }
    }
}

/// A simulation of the blockchain mining game.
///
/// ## Details
/// [Miner::get_action] is called on each [Miner] instance based on their
/// given order.
#[derive(Debug, Clone)]
pub struct Simulation {
    initial_blockchain: Option<Blockchain>,
    miners: Vec<Box<dyn Miner>>,
    power_dist: PowerDistribution,
    rounds: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("invalid mining power distribution")]
    PowerDistributionError(#[from] PowerDistributionError),
    #[error("could not create rand::distributions::WeightedIndex")]
    WeightedIndexError(#[from] WeightedError),
    #[error("block could not be published")]
    BlockPublishingError(#[from] BlockPublishingError),
}

impl Simulation {
    /// Executes the configured simulation.
    fn run(self) -> Result<SimulationOutput, SimulationError> {
        let Simulation {
            initial_blockchain,
            mut miners,
            power_dist,
            rounds,
        } = self;

        let mut rng = rand::thread_rng();
        let power_values = power_dist.values(miners.len())?;
        let gamma = WeightedIndex::new(power_values)?;

        let mut blockchain = initial_blockchain.unwrap_or_default();
        let mut blocks_by_miner: HashMap<MinerID, Vec<_>> = HashMap::new();
        for r in 1..=self.rounds {
            let proposer = gamma.sample(&mut rng) + 1;

            for m in miners.iter_mut() {
                let miner = m.id();
                let block = if proposer == miner { Some(r) } else { None };

                match m.get_action(&blockchain, block) {
                    Action::Wait => (),
                    Action::Publish(block) => {
                        blocks_by_miner
                            .entry(miner)
                            .or_default()
                            .push(block.id);
                        blockchain.publish(block)?;
                    }
                    Action::PublishSet(blocks) => {
                        for block in blocks {
                            blocks_by_miner
                                .entry(miner)
                                .or_default()
                                .push(block.id);
                            blockchain.publish(block)?;
                        }
                    }
                }
            }
        }

        let longest_chain = blockchain.longest_chain();
        Ok(SimulationOutput {
            blockchain,
            blocks_by_miner,
            longest_chain,
            miners,
            power_dist,
            rounds,
        })
    }
}
