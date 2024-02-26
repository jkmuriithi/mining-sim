/*!
Building/running simulations and analyzing the resulting data
*/

use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

use rand::distributions::{Distribution, WeightedError, WeightedIndex};
use rayon::prelude::*;

use crate::{
    blockchain::{BlockId, BlockPublishingError, Blockchain},
    miner::{Action, Miner, MinerId},
    power_dist::{PowerDistribution, PowerDistributionError, PowerValue},
    results::ResultsBuilder,
};

/// Builds up a set of simulations based on the configuration parameters.
#[derive(Debug, Default)]
pub struct SimulationBuilder {
    blockchain: Option<Blockchain>,
    power_dists: Vec<PowerDistribution>,
    repeat_all: Option<NonZeroUsize>,
    rounds: Option<NonZeroUsize>,
    miners: Vec<Box<dyn Miner>>,
    curr_miner_id: MinerId,
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationBuildError {
    #[error("no miners were added")]
    NoMinersGiven,
    #[error("number of simulation rounds must be greater than 0")]
    ZeroRounds,
    #[error("cannot repeat simulations 0 times")]
    ZeroRepeats,
    #[error("invalid mining power distribution")]
    PowerDistributionError(#[from] PowerDistributionError),
}

impl SimulationBuilder {
    /// Create a new [`SimulationBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `miner` to the simulation.
    pub fn add_miner<M: Miner + 'static>(mut self, mut miner: M) -> Self {
        miner.set_id(self.curr_miner_id);

        assert_eq!(
            self.curr_miner_id,
            miner.id(),
            "Miner {} method .id() does not return assigned MinerId",
            miner.name(),
        );

        self.miners.push(Box::new(miner));
        self.curr_miner_id.0 += 1;

        self
    }

    /// Run each configured simulation `num` times.
    pub fn repeat_all(mut self, num: usize) -> Self {
        self.repeat_all = NonZeroUsize::new(num);

        self
    }

    /// Set the initial blockchain state used in the simulation.
    /// [`Blockchain::default`] is used otherwise.
    pub fn blockchain(mut self, chain: Blockchain) -> Self {
        self.blockchain = Some(chain);

        self
    }

    /// Set the number of rounds the simulation will last for (default 1).
    pub fn rounds(mut self, rounds: usize) -> Self {
        self.rounds = NonZeroUsize::new(rounds);

        self
    }

    /// Run the simulation using the specified mining power distribution.
    pub fn power_dist(mut self, dist: PowerDistribution) -> Self {
        self.power_dists.push(dist);

        self
    }

    /// Run the simulation using the mining power distribution described by
    /// `values`.
    pub fn power_values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = PowerValue>,
    {
        let dist = values.into_iter().collect();
        self.power_dists.push(PowerDistribution::SetValues(dist));

        self
    }

    /// Run the simulation such that mining power is equally distributed
    /// between all miners (this is the default behavior).
    pub fn equal_power(mut self) -> Self {
        self.power_dists.push(PowerDistribution::Equal);

        self
    }

    /// Run the simulation such that the mining power of the given miner is
    /// `value`, and mining power is distributed equally between all other
    /// miners. `miner` is a 1-based index over the miners that are added to
    /// this [`SimulationBuilder`], in the order of addition.
    pub fn miner_power(mut self, miner: MinerId, value: PowerValue) -> Self {
        self.power_dists.push(PowerDistribution::SetMiner(miner, value));

        self
    }

    /// Call [`SimulationBuilder::miner_power`] once for each element of
    /// `values`.
    pub fn miner_power_iter<I>(mut self, miner: MinerId, values: I) -> Self
    where
        I: IntoIterator<Item = PowerValue>,
    {
        for val in values {
            self.power_dists.push(PowerDistribution::SetMiner(miner, val));
        }

        self
    }

    /// Create a [`SimulationGroup`] from the specified parameters.
    pub fn build(self) -> Result<SimulationGroup, SimulationBuildError> {
        use SimulationBuildError::*;

        let SimulationBuilder {
            blockchain,
            miners,
            mut power_dists,
            repeat_all,
            rounds,
            ..
        } = self;

        if miners.is_empty() {
            return Err(NoMinersGiven);
        }

        if power_dists.is_empty() {
            power_dists.push(PowerDistribution::Equal);
        }

        for power_dist in power_dists.iter() {
            power_dist.validate(miners.len())?;
        }

        let repeat_all = repeat_all.unwrap_or(NonZeroUsize::new(1).unwrap());
        let rounds = rounds.unwrap_or(NonZeroUsize::new(1).unwrap());

        Ok(SimulationGroup {
            blockchain,
            miners,
            power_dists,
            repeat_all,
            rounds,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::miner::honest::Honest;

    use super::SimulationBuilder;

    #[test]
    fn example_build() {
        SimulationBuilder::new()
            .add_miner(Honest::new())
            .build()
            .expect("valid simulation build");
    }
}

/// Container for a group of simulations which run on the same set of miners.
#[derive(Debug, Clone)]
pub struct SimulationGroup {
    blockchain: Option<Blockchain>,
    miners: Vec<Box<dyn Miner>>,
    power_dists: Vec<PowerDistribution>,
    repeat_all: NonZeroUsize,
    rounds: NonZeroUsize,
}

impl SimulationGroup {
    /// Returns the builder for this struct.
    pub fn builder() -> SimulationBuilder {
        SimulationBuilder::new()
    }

    /// Runs all configured simulations in parallel using [`rayon`].
    pub fn run_all(self) -> Result<ResultsBuilder, SimulationError> {
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

        Ok(ResultsBuilder::new(outputs?, repeat_all))
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
    pub blocks_by_miner: HashMap<MinerId, Vec<BlockId>>,
    pub blocks_published: usize,
    pub longest_chain: HashSet<BlockId>,
    pub miners: HashMap<MinerId, String>,
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
        let Simulation { mut blockchain, mut miners, power_dist, rounds } =
            self;

        let mut blocks_by_miner: HashMap<_, Vec<_>> = HashMap::new();

        // Safety: power distributions are validated during the simulation
        // build process
        let power_values = unsafe { power_dist.values_unchecked(miners.len()) };
        let gamma = WeightedIndex::new(power_values)?
            .sample_iter(rand::thread_rng())
            .enumerate()
            .map(|(round, proposer)| (round + 1, MinerId(proposer + 1)))
            .take(self.rounds);

        for (round, proposer) in gamma {
            for m in miners.iter_mut() {
                let miner_id = m.id();

                let block_mined =
                    (proposer == miner_id).then_some(BlockId(round));

                let blocks_published =
                    match m.get_action(&blockchain, block_mined) {
                        Action::Wait => vec![],
                        Action::Publish(block) => vec![block],
                        Action::PublishSet(blocks) => blocks,
                    };

                for block in blocks_published {
                    assert_eq!(
                        block.miner_id, miner_id,
                        "Miner {} published block with wrong MinerId",
                        miner_id
                    );

                    blocks_by_miner.entry(miner_id).or_default().push(block.id);
                    blockchain.publish(block)?;
                }
            }
        }

        let blocks_published = blockchain.num_blocks();
        let longest_chain = HashSet::from_iter(blockchain.longest_chain());
        let miners = miners.into_iter().map(|m| (m.id(), m.name())).collect();

        Ok(SimulationOutput {
            blocks_by_miner,
            blocks_published,
            longest_chain,
            miners,
            power_dist,
            rounds,
        })
    }
}
