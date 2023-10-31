use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use crate::{
    block::BlockID,
    blockchain::Blockchain,
    miner::{Miner, MinerID},
};

/// Allows for analysis of results from a [Simulation](super::Simulation).
///
/// For analyzing built-in metrics, the appropriate methods and
/// [SimulationResults::build_data] can be used to build [SimulationData] for
/// CSV output. For custom measurements, the pub fields of this struct must be
/// accessed  manually.
pub struct SimulationResults {
    /// Number of simulation runs to be associated with a single data point.
    /// Must be at least 1.
    pub average_of: usize,
    /// Miners used in the corresponding simulation.
    pub miners: Vec<Miner>,
    /// Mining power distribution used in the corresponding simulation.
    pub miner_alphas: Vec<Vec<f64>>,
    /// Blocks mined by each miner in each run of the simulation.
    pub miner_blocks: Vec<HashMap<MinerID, Vec<BlockID>>>,
    /// Blockchains resulting from each run of the corresponding simulation.
    pub chains: Vec<Blockchain>,
    /// Accrued metrics
    metrics: Vec<VecDeque<Metric>>,
}

/// Metrics constructed within [SimulationResults].
#[derive(Debug, Clone)]
pub enum Metric {
    Title(String),
    Alpha(f64),
    Revenue(f64),
}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Metric::Title(s) => write!(f, "{}", s),
            Metric::Alpha(a) => write!(f, "{:.6}", a),
            Metric::Revenue(rev) => write!(f, "{:.6}", rev),
        }
    }
}

impl SimulationResults {
    pub fn new(
        average_of: usize,
        miners: Vec<Miner>,
        alphas: Vec<Vec<f64>>,
        blocks: Vec<HashMap<MinerID, Vec<BlockID>>>,
        chains: Vec<Blockchain>,
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
            average_of,
            miners,
            miner_alphas: alphas,
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
                .push_back(Title(format!("Miner {} Revenue", miner + 1)));
        }

        let mut run = 0;
        for row in 1..self.metrics.len() {
            let mut averages = vec![0.0; self.miners.len()];

            for _ in 0..self.average_of {
                let lc: HashSet<BlockID> =
                    HashSet::from_iter(self.chains[run].longest_chain());
                let lc_len = lc.len() as f64;

                for (i, miner) in self.miners.iter().enumerate() {
                    let id = miner.id();
                    let rev = self.miner_blocks[run].get(&id).map(|blocks| {
                        let mut mined = 0.0;
                        for block in blocks {
                            if lc.contains(block) {
                                mined += 1.0;
                            }
                        }
                        mined / lc_len
                    });
                    averages[i] += rev.unwrap_or(0.0);
                }

                run += 1;
            }

            for average in averages {
                self.metrics[row]
                    .push_back(Revenue(average / self.average_of as f64))
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
                        .push_front(Title(format!("Miner {} Alpha", miner + 1)))
                } else {
                    self.metrics[row]
                        .push_front(Alpha(self.miner_alphas[row - 1][miner]))
                }
            }
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
