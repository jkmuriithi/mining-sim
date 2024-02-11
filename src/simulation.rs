//! Building/running simulations and analyzing the resulting data

use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

use rand::{
    distributions::{WeightedError, WeightedIndex},
    prelude::Distribution,
};
use rayon::prelude::*;

use crate::{
    block::BlockID,
    blockchain::{BlockPublishingError, Blockchain},
    miner::{Action, Miner, MinerID},
    power_dist::{PowerDistribution, PowerDistributionError},
};

pub mod builder;
pub mod results;

pub use builder::{SimulationBuildError, SimulationBuilder};
pub use results::{OutputFormat, SimulationResults, SimulationResultsBuilder};

/// Container for a group of simulations which run on the same set of miners.
/// Simulations should be run using this struct's `run_all` method.
#[derive(Debug, Clone)]
pub struct SimulationGroup {
    blockchain: Option<Blockchain>,
    miners: Vec<Box<dyn Miner>>,
    power_dists: Vec<PowerDistribution>,
    repeat_all: NonZeroUsize,
    rounds: NonZeroUsize,
}

impl SimulationGroup {
    pub fn add(&mut self, power_dist: PowerDistribution) {
        self.power_dists.push(power_dist);
    }

    pub fn builder() -> SimulationBuilder {
        SimulationBuilder::new()
    }

    pub fn run_all(self) -> Result<SimulationResultsBuilder, SimulationError> {
        let SimulationGroup {
            blockchain,
            miners,
            power_dists,
            repeat_all,
            rounds,
        } = self;

        let blockchain = blockchain.unwrap_or_default();

        let sims: Vec<_> = power_dists
            .into_iter()
            .map(|power_dist| Simulation {
                blockchain: blockchain.clone(),
                miners: miners.clone(),
                power_dist,
                rounds: rounds.get(),
            })
            // Clone each simulation repeat_all times
            .flat_map(|sim| vec![sim; repeat_all.get()])
            .collect();

        let outputs: Result<_, _> =
            sims.into_par_iter().map(|sim| sim.run()).collect();

        Ok(SimulationResultsBuilder::new(outputs?, repeat_all))
    }
}

/// A simulation of the blockchain mining game.
///
/// # Details
/// [`Miner::get_action`] is called on each [`Miner`] instance based on their
/// given order.
#[derive(Debug, Clone)]
struct Simulation {
    blockchain: Blockchain,
    miners: Vec<Box<dyn Miner>>,
    power_dist: PowerDistribution,
    rounds: usize,
}

/// Contains the output data from a simulation.
#[derive(Debug, Clone)]
pub struct SimulationOutput {
    pub blockchain: Blockchain,
    pub blocks_by_miner: HashMap<MinerID, Vec<BlockID>>,
    pub longest_chain: HashSet<BlockID>,
    pub miners: Vec<Box<dyn Miner>>,
    pub power_dist: PowerDistribution,
    pub rounds: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("block could not be published")]
    BlockPublishingError(#[from] BlockPublishingError),
    #[error("invalid mining power distribution")]
    PowerDistributionError(#[from] PowerDistributionError),
    #[error("could not create rand::distributions::WeightedIndex")]
    WeightedIndexError(#[from] WeightedError),
}

impl Simulation {
    /// Executes the configured simulation.
    fn run(self) -> Result<SimulationOutput, SimulationError> {
        let Simulation {
            mut blockchain,
            mut miners,
            power_dist,
            rounds,
        } = self;

        let mut rng = rand::thread_rng();
        // Safety: power distributions are validated during the simulation
        // build process, and there's no other way a user can create this struct
        let power_values = unsafe { power_dist.values_unchecked(miners.len()) };
        let gamma = WeightedIndex::new(power_values)?;

        let mut blocks_by_miner: HashMap<MinerID, Vec<_>> = HashMap::new();
        for r in 1..=self.rounds {
            let proposer = gamma.sample(&mut rng) + 1;

            // Always iterate through miners in list order
            for m in miners.iter_mut() {
                let miner = m.id();
                let block_id = if proposer == miner { Some(r) } else { None };

                match m.get_action(&blockchain, block_id) {
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

        let longest_chain = HashSet::from_iter(blockchain.longest_chain());
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
