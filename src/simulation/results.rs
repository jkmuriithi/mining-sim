use std::{collections::BTreeSet, fmt::Display, num::NonZeroUsize};

use crate::{
    miner::MinerID, power_dist::PowerValue, simulation::SimulationOutput,
};

const FLOAT_PRECISION_DIGITS: usize = 6;

/// Builder for the [Results] struct. Typically produced by running a
/// [SimulationGroup](super::SimulationGroup)
#[derive(Debug, Clone)]
pub struct SimulationResultsBuilder {
    averaged: bool,
    columns: BTreeSet<ColumnType>,
    data: Vec<SimulationOutput>,
    format: OutputFormat,
    repeat_all: NonZeroUsize,
}

impl SimulationResultsBuilder {
    pub(super) fn new(
        data: Vec<SimulationOutput>,
        repeat_all: NonZeroUsize,
    ) -> Self {
        Self {
            data,
            repeat_all,
            ..Default::default()
        }
    }

    pub fn averaged(mut self) -> Self {
        self.averaged = true;

        self
    }

    pub fn with_longest_chain_length(mut self) -> Self {
        self.columns.insert(ColumnType::LongestChainLength);

        self
    }

    pub fn with_miner_names(mut self) -> Self {
        let num_miners = self.data[0].miners.len();
        for miner_id in 1..=num_miners {
            self.columns.insert(ColumnType::MinerStrategyName(miner_id));
        }

        self
    }

    pub fn with_revenue(mut self) -> Self {
        let num_miners = self.data[0].miners.len();
        for miner_id in 1..=num_miners {
            self.columns.insert(ColumnType::MinerRevenue(miner_id));
        }

        self
    }

    pub fn with_rounds(mut self) -> Self {
        self.columns.insert(ColumnType::Rounds);

        self
    }

    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;

        self
    }

    pub fn build(self) -> SimulationResults {
        let SimulationResultsBuilder {
            averaged,
            mut columns,
            data,
            format,
            repeat_all,
        } = self;

        let num_miners = data[0].miners.len();
        for miner_id in 1..=num_miners {
            columns.insert(ColumnType::MiningPower(miner_id));
        }

        let rows = if averaged {
            data.chunks(repeat_all.get())
                .map(|sim_outputs| {
                    columns
                        .iter()
                        .map(|col_type| {
                            col_type.get_averaged_value(sim_outputs)
                        })
                        .collect()
                })
                .collect()
        } else {
            data.iter()
                .map(|sim_output| {
                    columns
                        .iter()
                        .map(|col_type| col_type.get_value(sim_output))
                        .collect()
                })
                .collect()
        };

        SimulationResults {
            format,
            columns,
            rows,
        }
    }
}

impl Default for SimulationResultsBuilder {
    fn default() -> Self {
        Self {
            averaged: Default::default(),
            columns: Default::default(),
            data: Default::default(),
            format: Default::default(),
            repeat_all: NonZeroUsize::new(1).unwrap(),
        }
    }
}

pub struct SimulationResults {
    pub format: OutputFormat,
    columns: BTreeSet<ColumnType>,
    rows: Vec<Vec<ColumnValue>>,
}

impl SimulationResults {
    pub const SEPARATOR_VERTICAL: char = '|';
    pub const SEPARATOR_HORIZONTAL: char = '-';
}

impl Display for SimulationResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            OutputFormat::CSV => {
                let titles: Vec<_> =
                    self.columns.iter().map(ToString::to_string).collect();

                writeln!(f, "{}", titles.join(","))?;

                for row in self.rows.iter() {
                    let row: Vec<_> =
                        row.iter().map(ToString::to_string).collect();

                    writeln!(f, "{}", row.join(","))?;
                }

                Ok(())
            }
            OutputFormat::PrettyPrint => {
                let titles: Vec<_> =
                    self.columns.iter().map(ToString::to_string).collect();
                let column_widths: Vec<_> =
                    titles.iter().map(String::len).collect();

                for title in titles {
                    write!(f, " {} ", title)?;
                    write!(f, "{}", Self::SEPARATOR_VERTICAL)?;
                }
                writeln!(f)?;

                let total_width = column_widths.iter().map(|x| x + 3).sum();
                for _ in 0..total_width {
                    write!(f, "{}", Self::SEPARATOR_HORIZONTAL)?;
                }

                for row in self.rows.iter() {
                    writeln!(f)?;

                    for (i, val) in row.iter().enumerate() {
                        let val = val.to_string();

                        write!(f, " {:1$} ", val, column_widths[i])?;
                        write!(f, "{}", Self::SEPARATOR_VERTICAL)?;
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum OutputFormat {
    CSV,
    #[default]
    PrettyPrint,
}

/// Type of column that can appear in a data table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColumnType {
    // Variant order determines the order of the data tables:
    // https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html#derivable
    MinerStrategyName(MinerID),
    MiningPower(MinerID),
    MinerRevenue(MinerID),
    Rounds,
    LongestChainLength,
}

#[inline]
fn revenue_of(miner_id: MinerID, data: &SimulationOutput) -> f64 {
    let mut revenue = 0.0;
    for block_id in data.longest_chain.iter() {
        if data.blockchain[block_id].block.miner_id == miner_id {
            revenue += 1.0;
        }
    }

    revenue / data.longest_chain.len() as f64
}

impl ColumnType {
    fn get_value(&self, data: &SimulationOutput) -> ColumnValue {
        match &self {
            Self::MinerStrategyName(miner_id) => {
                let name = data.miners[*miner_id - 1].name();

                ColumnValue::MinerStrategyName(name)
            }
            Self::MiningPower(miner_id) => {
                let power = data
                    .power_dist
                    .power_of(*miner_id, data.miners.len())
                    .expect("valid power distribution");

                ColumnValue::MiningPower(power)
            }
            Self::MinerRevenue(miner_id) => {
                let revenue = revenue_of(*miner_id, data);

                ColumnValue::MinerRevenue(revenue)
            }
            Self::Rounds => {
                let rounds = data.rounds;

                ColumnValue::Rounds(rounds)
            }
            Self::LongestChainLength => {
                let length = data.longest_chain.len() as f64;

                ColumnValue::LongestChainLength(length)
            }
        }
    }

    pub fn get_averaged_value(&self, data: &[SimulationOutput]) -> ColumnValue {
        match &self {
            Self::MinerRevenue(miner_id) => {
                let mut revenue = 0.0;
                for sim_output in data {
                    revenue += revenue_of(*miner_id, sim_output);
                }
                revenue /= data.len() as f64;

                ColumnValue::MinerRevenue(revenue)
            }
            Self::LongestChainLength => {
                let mut length = 0.0;
                for sim_output in data {
                    length += sim_output.longest_chain.len() as f64;
                }
                length /= data.len() as f64;

                ColumnValue::LongestChainLength(length)
            }
            // use get_value on the first element for invariant fields
            Self::MinerStrategyName(_)
            | Self::MiningPower(_)
            | Self::Rounds => self.get_value(&data[0]),
        }
    }
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::MinerStrategyName(miner_id) => {
                write!(f, "Miner {} Strategy", miner_id)
            }
            Self::MiningPower(miner_id) => {
                write!(f, "Miner {} Power", miner_id)
            }
            Self::MinerRevenue(miner_id) => {
                write!(f, "Miner {} Revenue", miner_id)
            }
            Self::Rounds => {
                write!(f, "Simulated Rounds")
            }
            Self::LongestChainLength => {
                write!(f, "Longest Chain Length")
            }
        }
    }
}

/// Value which corresponds to a [ColumnType].
#[derive(Debug, Clone)]
pub enum ColumnValue {
    MinerStrategyName(String),
    MiningPower(PowerValue),
    MinerRevenue(f64),
    Rounds(usize),
    LongestChainLength(f64),
}

impl Display for ColumnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::MinerStrategyName(name) => {
                write!(f, "{}", name)
            }
            Self::MiningPower(power) => {
                write!(f, "{:.1$}", power, FLOAT_PRECISION_DIGITS)
            }
            Self::MinerRevenue(revenue) => {
                write!(f, "{:.1$}", revenue, FLOAT_PRECISION_DIGITS)
            }
            Self::Rounds(rounds) => {
                write!(f, "{}", rounds)
            }
            Self::LongestChainLength(length) => {
                write!(f, "{:.1$}", length, FLOAT_PRECISION_DIGITS)
            }
        }
    }
}
