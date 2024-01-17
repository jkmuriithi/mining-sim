use crate::{
    blockchain::Blockchain,
    miner::{Miner, MinerID},
};

use super::{
    power_dist::PowerValue, PowerDistribution, PowerDistributionError,
    Simulation,
};

/// Builds a [Simulation].
#[derive(Debug, Default)]
pub struct SimulationBuilder {
    pub average_of: Option<usize>,
    pub rounds: Option<usize>,
    pub power_dists: Vec<PowerDistribution>,
    pub initial_blockchain: Option<Blockchain>,
    miners: Vec<Box<dyn Miner>>,
    last_assigned_miner_id: MinerID,
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationBuildError {
    #[error("no miners were added or created")]
    NoMinersGiven,
    #[error("number of simulation rounds must be greater than 0")]
    ZeroRounds,
    #[error("cannot take the average of 0 simulation runs")]
    AverageOfZero,
    #[error(transparent)]
    PowerDistributionError(#[from] PowerDistributionError),
}

impl SimulationBuilder {
    /// Creates a new [SimulationBuilder].
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

    /// The simulation will run `num` times and each result metric will be
    /// averaged over the set of `num` runs.
    pub fn taking_average_of(mut self, num: usize) -> Self {
        self.average_of = Some(num);

        self
    }

    /// Sets the initial blockchain state used in the simulation.
    /// ([Blockchain::default] used otherwise).
    pub fn with_blockchain(mut self, chain: Blockchain) -> Self {
        self.initial_blockchain = Some(chain);

        self
    }

    /// Sets the number of rounds the simulation will last for (default 1).
    pub fn with_rounds(mut self, rounds: usize) -> Self {
        self.rounds = Some(rounds);

        self
    }

    /// Run the simulation such that the respective mining power of all miners
    /// is equal to what's specified by `values`.
    pub fn with_power_dist<I>(mut self, values: I) -> Self
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

    /// Call `SimulationBuilder::with_miner_power` once for each element of
    /// `values`.
    pub fn with_miner_power_iter<I>(mut self, miner: MinerID, values: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        for val in values {
            self.power_dists
                .push(PowerDistribution::SetMiner(miner, val));
        }

        self
    }

    /// Creates a [Simulation] from the specified parameters.
    pub fn build(self) -> Result<Simulation, SimulationBuildError> {
        use SimulationBuildError::*;

        let SimulationBuilder {
            average_of,
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

        let average_of = match average_of {
            Some(0) => return Err(AverageOfZero),
            Some(x) => x,
            None => 1,
        };
        let rounds = match rounds {
            Some(0) => return Err(ZeroRounds),
            Some(x) => x,
            None => 1,
        };
        let n = miners.len();

        let mut dists = vec![];
        for power_dist in power_dists {
            match power_dist.values(n) {
                Ok(dist) => dists.push(dist),
                Err(pde) => return Err(pde.into()),
            }
        }

        Ok(Simulation {
            average_of,
            rounds,
            miners,
            power_dists: dists,
            initial_blockchain,
        })
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
