use crate::{
    blockchain::Blockchain,
    miner::{Miner, MinerID},
};

use super::{
    power_dist::PowerValue, PowerDistribution, PowerDistributionError,
    Simulation, SimulationGroup,
};

/// Builds up a set of simulations based on the configuration parameters.
/// TODO: Explain methods and write example code.
#[derive(Debug, Default)]
pub struct SimulationBuilder {
    pub initial_blockchain: Option<Blockchain>,
    pub power_dists: Vec<PowerDistribution>,
    pub repeat_all: Option<usize>,
    pub rounds: Option<usize>,
    last_assigned_miner_id: MinerID,
    miners: Vec<Box<dyn Miner>>,
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
    /// Create a new [SimulationBuilder].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `miner` to the simulation.
    pub fn add_miner<M: Miner + 'static>(mut self, mut miner: M) -> Self {
        miner.set_id(self.last_assigned_miner_id + 1);
        self.miners.push(Box::new(miner));

        self.last_assigned_miner_id += 1;
        self
    }

    /// Run each configured simulation `num` times.
    pub fn repeat_all(mut self, num: usize) -> Self {
        self.repeat_all = Some(num);

        self
    }

    /// Set the initial blockchain state used in the simulation.
    /// ([Blockchain::default] used otherwise).
    pub fn with_blockchain(mut self, chain: Blockchain) -> Self {
        self.initial_blockchain = Some(chain);

        self
    }

    /// Set the number of rounds the simulation will last for (default 1).
    pub fn with_rounds(mut self, rounds: usize) -> Self {
        self.rounds = Some(rounds);

        self
    }

    /// Run the simulation using the specified mining power distribution.
    pub fn with_power_dist<I>(mut self, dist: PowerDistribution) -> Self {
        self.power_dists.push(dist);

        self
    }

    /// Run the simulation using the mining power distribution described by
    /// `values`.
    pub fn with_power_values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = PowerValue>,
    {
        let dist = values.into_iter().collect();
        self.power_dists.push(PowerDistribution::SetValues(dist));

        self
    }

    /// Run the simulation such that mining power is equally distributed
    /// between all miners (this is the default behavior).
    pub fn with_equal_power(mut self) -> Self {
        self.power_dists.push(PowerDistribution::Equal);

        self
    }

    /// Run the simulation such that the mining power of the given miner is
    /// `value`, and mining power is distributed equally between all other
    /// miners. `miner` is a 1-based index over the miners that are added to
    /// this [SimulationBuilder], in the order of addition.
    pub fn with_miner_power(
        mut self,
        miner: MinerID,
        value: PowerValue,
    ) -> Self {
        self.power_dists
            .push(PowerDistribution::SetMiner(miner, value));

        self
    }

    /// Call [SimulationBuilder::with_miner_power] once for each element of
    /// `values`.
    pub fn with_miner_power_iter<I>(mut self, miner: MinerID, values: I) -> Self
    where
        I: IntoIterator<Item = PowerValue>,
    {
        for val in values {
            self.power_dists
                .push(PowerDistribution::SetMiner(miner, val));
        }

        self
    }

    /// Creates a [SimulationGroup] from the specified parameters.
    pub fn build(self) -> Result<SimulationGroup, SimulationBuildError> {
        use SimulationBuildError::*;

        let SimulationBuilder {
            repeat_all: repeat_each,
            rounds,
            mut power_dists,
            initial_blockchain,
            miners,
            ..
        } = self;

        if miners.is_empty() {
            return Err(NoMinersGiven);
        }
        if power_dists.is_empty() {
            power_dists.push(PowerDistribution::Equal);
        }

        let repeat_each = match repeat_each {
            Some(0) => return Err(ZeroRepeats),
            Some(x) => x,
            None => 1,
        };
        let mut group = SimulationGroup::new(repeat_each);

        let rounds = match rounds {
            Some(0) => return Err(ZeroRounds),
            Some(x) => x,
            None => 1,
        };

        for power_dist in power_dists {
            power_dist.validate(miners.len())?;
            group.add(Simulation {
                initial_blockchain: initial_blockchain.clone(),
                miners: miners.clone(),
                power_dist,
                rounds,
            })
        }

        Ok(group)
    }
}

#[cfg(test)]
mod tests {
    use crate::miner::Honest;

    use super::SimulationBuilder;

    #[test]
    fn example_build() {
        SimulationBuilder::new()
            .add_miner(Honest::new())
            .build()
            .expect("valid simulation build");
    }
}
