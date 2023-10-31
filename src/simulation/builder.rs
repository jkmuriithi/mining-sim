use thiserror::Error;

use crate::{
    blockchain::Blockchain,
    miner::{Miner, MinerID},
};

use super::Simulation;

/// Allowable difference between the sum of a mining power distribution and 1.
const F64_MARGIN: f64 = 1e-6;

/// Builds a [Simulation].
#[derive(Debug)]
pub struct SimulationBuilder {
    average_of: Option<usize>,
    rounds: Option<u64>,
    miners: Vec<Miner>,
    alpha_dists: Vec<AlphaDist>,
    /// Incremented as miners are added
    curr_miner_id: u64,
    chain: Option<Blockchain>,
}

/// Determines how alpha (mining power) is distributed between miners in the
/// configured simulation.
#[derive(Debug, Default, Clone)]
pub enum AlphaDist {
    /// Weights each miner equally.
    #[default]
    Equal,
    /// Sets the specified miner's alpha to the given float in \[0, 1\], with
    /// mining power distributed equally between all other miners.
    SetMinerAlpha(MinerID, f64),
    /// Sets all miners' alphas to the values specified in the given vector.
    SetMinerAlphas(Vec<f64>),
}

#[derive(Debug, Error)]
pub enum SimulationBuildError {
    #[error("no miners were added or created")]
    NoMinersGiven,
    #[error("no miner exists with specified ID")]
    MinerNotFound,
    #[error("alpha distribution size does not match number of given miners")]
    WrongAlphaDistSize,
    #[error("cannot set alpha of a single miner")]
    SingleMinerSetAlpha,
}

impl SimulationBuilder {
    /// Creates a new [SimulationBuilder].
    pub fn new() -> Self {
        SimulationBuilder {
            average_of: None,
            rounds: None,
            miners: vec![],
            alpha_dists: vec![],
            curr_miner_id: 1,
            chain: None,
        }
    }

    /// Creates a new miner and adds it to the simulation, setting its [MinerID]
    /// to be (1 + the number of miners already added).    
    pub fn add_miner<M: Into<Miner>>(mut self, miner: M) -> Self {
        let mut miner = miner.into();
        miner.set_id(self.curr_miner_id);
        self.miners.push(miner);
        self.curr_miner_id += 1;

        self
    }

    /// Set the number of data points collected and averaged for the simulation
    /// run (default 1).
    pub fn average_of(mut self, size: usize) -> Self {
        if self.average_of.is_some() {
            panic!("average_of cannot be set twice");
        }
        if size == 0 {
            panic!("average size cannot be zero");
        }
        self.average_of = Some(size);

        self
    }

    /// Sets the initial blockchain state used in the simulation.
    /// ([Blockchain::default] used otherwise).
    pub fn blockchain(mut self, chain: Blockchain) -> Self {
        if self.chain.is_some() {
            panic!("blockchain cannot be set twice");
        }
        self.chain = Some(chain);

        self
    }

    /// Sets the number of rounds the simulation will last for (default 1).
    pub fn rounds(mut self, rounds: u64) -> Self {
        if self.rounds.is_some() {
            panic!("number of rounds cannot be set twice");
        }
        if rounds == 0 {
            panic!("number of rounds cannot be zero");
        }
        self.rounds = Some(rounds);

        self
    }

    /// Run the simulation such that the respective mining power of all miners
    /// is equal to what's specified in `arr`.
    pub fn with_alphas<const N: usize>(mut self, arr: [f64; N]) -> Self {
        if f64::abs(arr.iter().sum::<f64>() - 1.0) > F64_MARGIN {
            panic!("alphas must sum to 1.0");
        }
        self.alpha_dists
            .push(AlphaDist::SetMinerAlphas(arr.to_vec()));

        self
    }

    /// Run the simulation such that mining power is equally distributed
    /// between all miners (this is the default behavior).
    pub fn with_equal_alphas(mut self) -> Self {
        self.alpha_dists.push(AlphaDist::Equal);

        self
    }

    /// Run the simulation such that the mining power of the specified miner is
    /// equal to `alpha`, and mining power is distributed equally between all
    /// other miners. `miner` is a 1-based index over the miners that are
    /// added to this [SimulationBuilder], in the order of addition.
    pub fn with_miner_alpha(mut self, miner: u64, alpha: f64) -> Self {
        if miner == 0 {
            panic!("miner indices start at 1");
        }
        if !(0.0..=1.0).contains(&alpha) {
            panic!("alpha must be in [0.0, 1.0]");
        }
        self.alpha_dists
            .push(AlphaDist::SetMinerAlpha(miner.into(), alpha));

        self
    }

    /// Creates a [Simulation] from the specified parameters.
    pub fn build(mut self) -> Result<Simulation, SimulationBuildError> {
        use SimulationBuildError::*;

        if self.miners.is_empty() {
            return Err(NoMinersGiven);
        }
        if self.alpha_dists.is_empty() {
            self.alpha_dists.push(AlphaDist::Equal);
        }

        let zero = MinerID::from(0);
        let max_id = MinerID::from(self.miners.len() as u64);
        let n = self.miners.len();

        // Validate alpha distrbutions
        for dist in &self.alpha_dists {
            match dist {
                AlphaDist::SetMinerAlpha(id, _) => {
                    if self.miners.len() == 1 {
                        return Err(SingleMinerSetAlpha);
                    }

                    let id = *id;
                    if id == zero || id > max_id {
                        return Err(MinerNotFound);
                    }
                }
                AlphaDist::SetMinerAlphas(alphas) => {
                    if alphas.len() != n {
                        return Err(WrongAlphaDistSize);
                    }
                }
                _ => (),
            }
        }

        let average_of = self.average_of.unwrap_or(1);
        let rounds = self.rounds.unwrap_or(1);
        let chain = self.chain.unwrap_or_default();

        let miner_alphas = self
            .alpha_dists
            .into_iter()
            .map(|dist| match dist {
                AlphaDist::Equal => {
                    vec![1.0 / n as f64; n]
                }
                AlphaDist::SetMinerAlpha(id, alpha) => {
                    let other =
                        (1.0 - alpha) / n.checked_sub(1).unwrap() as f64;
                    (0..n)
                        .map(|i| {
                            if id == MinerID::from(i as u64) {
                                alpha
                            } else {
                                other
                            }
                        })
                        .collect()
                }
                AlphaDist::SetMinerAlphas(alphas) => alphas,
            })
            .collect();

        Ok(Simulation {
            average_of,
            rounds,
            miners: self.miners,
            miner_alphas,
            chain,
        })
    }
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a new [SimulationBuilder].
pub fn create() -> SimulationBuilder {
    SimulationBuilder::new()
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
