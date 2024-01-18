use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use crate::{
    block::BlockID,
    blockchain::Blockchain,
    miner::{Miner, MinerID},
};

use super::{power_dist::PowerValue, PowerDistribution};

/// Contains the output data from a [Simulation](super::Simulation).
#[derive(Debug, Clone)]
pub struct SimulationOutput {
    pub blockchain: Blockchain,
    pub blocks_by_miner: HashMap<MinerID, Vec<BlockID>>,
    pub miners: Vec<Box<dyn Miner>>,
    pub power_dist: PowerDistribution,
    pub rounds: usize,
}

/// Allows for analysis of results from a [Simulation](super::Simulation).
///
/// For analyzing built-in metrics, the appropriate methods and
/// [SimulationResults::build_data] can be used to build [SimulationData] for
/// CSV output. For custom measurements, the pub fields of this struct must be
/// accessed manually.
pub struct SimulationResults {
    /// The number of rounds in each simulation run.
    pub rounds: usize,
    /// Number of simulation runs to be associated with a single data point.
    /// Must be at least 1.
    pub average_of: usize,
    /// Blockchains resulting from each run of the corresponding simulation.
    pub chains: Vec<Blockchain>,
    /// Miners used in the corresponding simulation.
    pub miners: Vec<Box<dyn Miner>>,
    /// Mining power distribution used in the corresponding simulation.
    pub power_dists: Vec<Vec<PowerValue>>,
    /// Blocks published by each miner in each run of the simulation, in the
    /// order that they were published.
    pub miner_blocks: Vec<HashMap<MinerID, Vec<BlockID>>>,
    /// Accrued metrics
    metrics: Vec<VecDeque<Metric>>,
}

/// Metric types constructed within [SimulationResults].
#[derive(Debug, Clone)]
pub enum Metric {
    Text(String),
    Int(usize),
    Float(PowerValue),
}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Metric::Text(t) => write!(f, "{}", t),
            Metric::Int(i) => write!(f, "{}", i),
            Metric::Float(fl) => write!(f, "{:.6}", fl),
        }
    }
}

impl SimulationResults {
    pub fn new(
        rounds: usize,
        average_of: usize,
        chains: Vec<Blockchain>,
        miners: Vec<Box<dyn Miner>>,
        alphas: Vec<Vec<PowerValue>>,
        blocks: Vec<HashMap<MinerID, Vec<BlockID>>>,
    ) -> Self {
        assert!(
            average_of > 0,
            "tried to build results with average_of == 0"
        );
        assert!(!chains.is_empty(), "chains vec is empty");
        assert!(
            chains.len() % average_of == 0,
            "chains.len() and average_of do not agree"
        );

        let metrics = vec![VecDeque::new(); alphas.len() + 1];
        SimulationResults {
            rounds,
            average_of,
            miners,
            power_dists: alphas,
            miner_blocks: blocks,
            chains,
            metrics,
        }
    }

    /// Include [Metric::Revenue] in the [SimulationData] built with this
    /// struct.
    pub fn calculate_revenue(mut self) -> Self {
        use Metric::*;

        for miner in 0..self.miners.len() {
            self.metrics[0]
                .push_back(Text(format!("Miner {} Revenue", miner + 1)));
        }

        let mut run = 0;
        for row in 1..self.metrics.len() {
            let mut averages = vec![0.0; self.miners.len()];

            for _ in 0..self.average_of {
                let lc: HashSet<BlockID> =
                    HashSet::from_iter(self.chains[run].longest_chain());
                let lc_len = lc.len() as PowerValue;

                for (i, miner) in self.miners.iter().enumerate() {
                    let id = miner.id();
                    let revenue =
                        self.miner_blocks[run].get(&id).map(|blocks| {
                            let mut mined = 0.0;
                            for block in blocks {
                                if lc.contains(block) {
                                    mined += 1.0;
                                }
                            }
                            mined / lc_len
                        });
                    averages[i] += revenue.unwrap_or(0.0);
                }

                run += 1;
            }

            for average in averages {
                self.metrics[row]
                    .push_back(Float(average / self.average_of as PowerValue))
            }
        }

        self
    }

    /// Create [SimulationData] from this struct.
    pub fn build_data(mut self) -> SimulationData {
        use Metric::*;

        // Insert miner alphas
        for row in 0..self.metrics.len() {
            for miner in (0..self.miners.len()).rev() {
                if row == 0 {
                    self.metrics[row]
                        .push_front(Text(format!("Miner {} Alpha", miner + 1)))
                } else {
                    self.metrics[row]
                        .push_front(Float(self.power_dists[row - 1][miner]))
                }
            }
        }

        // Add metadata
        self.metrics[0].push_back(Text("Averaged Runs Per Datum".into()));
        self.metrics[1].push_back(Int(self.average_of));
        self.metrics[0].push_back(Text("Number of Rounds Per Run".into()));
        self.metrics[1].push_back(Int(self.rounds));
        for miner in 0..self.miners.len() {
            self.metrics[0]
                .push_back(Text(format!("Miner {} Strategy", miner + 1)));
            self.metrics[1].push_back(Text(self.miners[miner].name()));
        }

        SimulationData {
            metrics: self.metrics.into_iter().map(|dq| dq.into()).collect(),
        }
    }
}

/// Data produced by running a [Simulation](super::Simulation)
/// with a certain alpha distribution.
pub struct SimulationData {
    /// Table of metrics
    metrics: Vec<Vec<Metric>>,
}

impl Display for SimulationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.metrics.len() {
            for j in 0..self.metrics[i].len() {
                write!(f, "{}", self.metrics[i][j])?;

                if j != self.metrics[i].len() - 1 {
                    write!(f, ",")?;
                }
            }

            if i != self.metrics.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
