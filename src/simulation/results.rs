use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
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
    pub longest_chain: Vec<BlockID>,
    pub miners: Vec<Box<dyn Miner>>,
    pub power_dist: PowerDistribution,
    pub rounds: usize,
}

#[derive(Debug, Clone)]
pub struct ResultsBuilder {
    // TODO: Implement averaged data (through some sort of O(1) SimulationOutput
    // comparison?)
    _averaged: bool,
    columns: BTreeSet<ColumnType>,
    data: Vec<SimulationOutput>,
    format: OutputFormat,
}

impl ResultsBuilder {
    pub fn new(data: Vec<SimulationOutput>) -> Self {
        Self {
            data,
            _averaged: Default::default(),
            columns: Default::default(),
            format: Default::default(),
        }
    }
}

pub struct Results {
    columns: BTreeSet<ColumnType>,
    values: Vec<Vec<ColumnValue>>,
}

#[derive(Debug, Clone, Default)]
enum OutputFormat {
    #[default]
    CSV,
    PrettyPrint,
}

/// Type of column that can appear in a data table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ColumnType {
    // Variant order determines the order of the data tables:
    // https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html#derivable
    MiningPower(MinerID),
    MinerStrategyName(MinerID),
    Rounds,
    MinerRevenue(MinerID),
    LongestChainLength,
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// Value which corresponds to a [ColumnType].
#[derive(Debug, Clone)]
enum ColumnValue {
    MiningPower(PowerValue),
    MinerStrategyName(String),
    Rounds(usize),
    MinerRevenue(f64),
    LongestChainLength(usize),
}

impl ColumnType {
    fn extract_value(&self, data: &SimulationOutput) -> ColumnValue {
        match &self {
            Self::MiningPower(miner_id) => {
                // FIXME: Switch to checked version?
                let power = data
                    .power_dist
                    .power_of_unchecked(*miner_id, data.miners.len());

                ColumnValue::MiningPower(power)
            }
            Self::MinerStrategyName(miner_id) => {
                let name = data.miners[*miner_id].name();

                ColumnValue::MinerStrategyName(name)
            }
            Self::Rounds => {
                let rounds = data.rounds;

                ColumnValue::Rounds(rounds)
            }
            Self::MinerRevenue(miner_id) => {
                let mut revenue = 0.0;
                for block_id in &data.longest_chain {
                    if data.blockchain[block_id].block.miner_id.eq(miner_id) {
                        revenue += 1.0;
                    }
                }
                revenue /= data.longest_chain.len() as f64;

                ColumnValue::MinerRevenue(revenue)
            }
            Self::LongestChainLength => {
                let length = data.longest_chain.len();

                ColumnValue::LongestChainLength(length)
            }
        }
    }
}

impl Display for ColumnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
