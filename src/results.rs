/*!
Defines data types which control the appearance of simulation results.

# Formatting Results


# Aggregating Results

# Examples

Creating [`SimulationResults`] after running a simulation group:
```
use mining_sim::{miner, Format, SimulationBuilder};

let sim = SimulationBuilder::new()
    .add_miner(miner::Honest::new())
    .add_miner(miner::Honest::new())
    .repeat_all(5)
    .power_values([0.1, 0.9])
    .build()
    .unwrap();

let results_builder = sim.run_all().unwrap();

let results = results_builder
    // Average results of repeated simulations
    .averaged()
    // Include the number of rounds run per simulation
    .rounds()
    // Output results as CSV
    .format(Format::CSV)
    .build();

println!("{}", results);
```
*/

use std::{collections::BTreeSet, fmt::Display, num::NonZeroUsize};

use rayon::prelude::*;

use crate::{
    miner::MinerID, power_dist::PowerValue, simulation::SimulationOutput,
    utils::WrapFunc, wrap,
};

/// Floating point precision of results data.
pub const FLOAT_PRECISION_DIGITS: usize = 6;

/// Builder for [`SimulationResults`]. Typically produced by running a
/// [`SimulationGroup`](super::SimulationGroup).
#[derive(Debug, Clone)]
pub struct SimulationResultsBuilder {
    averaged: bool,
    columns: BTreeSet<Column>,
    data: Vec<SimulationOutput>,
    format: Format,
    repeat_all: NonZeroUsize,
}

/// Describes the appearance of a [`SimulationResults`] table as given by its
/// [`Display`] implementation.
#[derive(Debug, Clone, Copy, Default)]
pub enum Format {
    /// Comma-separated, without extra whitespace.
    CSV,
    /// Human-readable.
    #[default]
    PrettyPrint,
}

impl SimulationResultsBuilder {
    /// Create a new [`SimulationResultsBuilder`].
    pub(crate) fn new(
        data: Vec<SimulationOutput>,
        repeat_all: NonZeroUsize,
    ) -> Self {
        Self {
            data,
            repeat_all,
            averaged: false,
            columns: BTreeSet::default(),
            format: Format::default(),
        }
    }

    /// Include the "Blocks Published", "Longest Chain Length",
    /// "Miner `X` Strategy Name", "Miner `X` Revenue", and "Simulated Rounds"
    /// columns.
    ///
    /// [`SimulationResultsBuilder::averaged`] must still be called separately
    /// to create averaged data.
    pub fn all(self) -> Self {
        self.blocks_published()
            .longest_chain_length()
            .strategy_names()
            .revenue()
            .rounds()
    }

    /// Average the results of repeated simulations and include the "Average Of"
    /// column in the results table.
    pub fn averaged(mut self) -> Self {
        if self.repeat_all.get() > 1 {
            self.averaged = true;
            self.columns.insert(Column::AverageOf);
        }

        self
    }

    /// Include the "Blocks Published" column in the results table.
    pub fn blocks_published(mut self) -> Self {
        self.columns.insert(Column::BlocksPublished);

        self
    }

    /// Include a column with title `title` which only contains the given
    /// value.
    pub fn constant<T>(mut self, title: T, value: f64) -> Self
    where
        T: Into<String>,
    {
        self.columns
            .insert(Column::Constant(wrap!(title, move |_| value)));

        self
    }

    /// Include the "Longest Chain Length" column in the results table.
    pub fn longest_chain_length(mut self) -> Self {
        self.columns.insert(Column::LongestChainLength);

        self
    }

    /// Use the mining power of the miner with ID `miner_id` as input to `func`,
    /// and present the output in a table column with the given title.
    pub fn mining_power_func<T, F>(
        mut self,
        miner_id: MinerID,
        title: T,
        func: F,
    ) -> Self
    where
        T: Into<String>,
        F: Fn(PowerValue) -> f64 + Send + Sync + 'static,
    {
        self.columns
            .insert(Column::MiningPowerFunction(miner_id, wrap!(title, func)));

        self
    }

    /// Include a "Miner `X` Strategy Name" column in the results table for each
    /// miner `X`.
    pub fn strategy_names(mut self) -> Self {
        let num_miners = self.data[0].miners.len();
        for miner_id in 1..=num_miners {
            self.columns.insert(Column::MinerStrategyName(miner_id));
        }

        self
    }

    /// Include a "Miner `X` Revenue" column in the results table for each
    /// miner `X`.
    pub fn revenue(mut self) -> Self {
        let num_miners = self.data[0].miners.len();
        for miner_id in 1..=num_miners {
            self.columns.insert(Column::MinerRevenue(miner_id));
        }

        self
    }

    /// Include the "Simulated Rounds" column in the results table.
    pub fn rounds(mut self) -> Self {
        self.columns.insert(Column::Rounds);

        self
    }

    /// Specify the [`Format`] of the results table.
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;

        self
    }

    /// Create new [`SimulationResults`].
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
            columns.insert(Column::MiningPower(miner_id));
        }

        let columns = Vec::from_iter(columns);
        let rows = if averaged {
            data.chunks(repeat_all.get())
                .map(|sim_outputs| {
                    columns
                        .par_iter()
                        .map(|col_type| col_type.get_average_value(sim_outputs))
                        .collect()
                })
                .collect()
        } else {
            data.iter()
                .map(|sim_output| {
                    columns
                        .par_iter()
                        .map(|col_type| col_type.get_value(sim_output))
                        .collect()
                })
                .collect()
        };

        SimulationResults {
            columns,
            format,
            rows,
        }
    }
}

/// Formatted results from the completion of a
/// [`SimulationGroup`](super::SimulationGroup). The results table is given by the
/// struct's [`Display`] implementation, as specified by its [`Format`].
pub struct SimulationResults {
    columns: Vec<Column>,
    format: Format,
    rows: Vec<Vec<ColumnValue>>,
}

impl SimulationResults {
    const SEPARATOR_VERTICAL: char = '|';
    const SEPARATOR_HORIZONTAL: char = '-';

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn set_format(&mut self, format: Format) {
        self.format = format;
    }
}

impl Display for SimulationResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let titles: Vec<_> = self
            .columns
            .iter()
            .map(|col_type| col_type.to_string())
            .collect();

        match self.format {
            Format::CSV => {
                write!(f, "{}", titles.join(","))?;

                for row in self.rows.iter() {
                    writeln!(f)?;

                    let row: Vec<_> =
                        row.iter().map(|val| val.to_string()).collect();

                    write!(f, "{}", row.join(","))?;
                }
            }
            Format::PrettyPrint => {
                let mut text_widths: Vec<_> =
                    titles.iter().map(|title| title.len()).collect();

                for row in self.rows.iter() {
                    for (i, val) in row.iter().enumerate() {
                        let val = val.to_string();
                        text_widths[i] = text_widths[i].max(val.len());
                    }
                }

                for (i, title) in titles.into_iter().enumerate() {
                    write!(
                        f,
                        " {:1$} {2}",
                        title,
                        text_widths[i],
                        Self::SEPARATOR_VERTICAL
                    )?;
                }
                writeln!(f)?;

                let total_width = text_widths.iter().map(|x| x + 3).sum();
                for _ in 0..total_width {
                    write!(f, "{}", Self::SEPARATOR_HORIZONTAL)?;
                }

                for row in self.rows.iter() {
                    writeln!(f)?;

                    for (i, val) in row.iter().enumerate() {
                        write!(
                            f,
                            " {:1$} {2}",
                            val.to_string(),
                            text_widths[i],
                            Self::SEPARATOR_VERTICAL
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Type of column that can appear in a data table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Column {
    // Variant order determines the order of columns in results tables:
    // https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html#derivable
    MinerStrategyName(MinerID),
    MiningPower(MinerID),
    MinerRevenue(MinerID),
    MiningPowerFunction(MinerID, WrapFunc<PowerValue, f64>),
    Constant(WrapFunc<(), f64>),
    Rounds,
    AverageOf,
    BlocksPublished,
    LongestChainLength,
}

/// Value which corresponds to a [`Column`].
#[derive(Debug, Clone)]
enum ColumnValue {
    MinerStrategyName(String),
    MiningPower(PowerValue),
    MinerRevenue(f64),
    MiningPowerFunction(f64),
    Constant(f64),
    Rounds(usize),
    AverageOf(usize),
    BlocksPublished(f64),
    LongestChainLength(f64),
}

#[inline]
fn revenue_of(miner_id: &MinerID, data: &SimulationOutput) -> f64 {
    let blocks = data
        .blocks_by_miner
        .get(miner_id)
        .map(|block_ids| {
            block_ids
                .iter()
                .filter(|&block_id| data.longest_chain.contains(block_id))
                .count() as f64
        })
        .unwrap_or_default();

    blocks / data.longest_chain.len() as f64
}

impl Column {
    fn get_value(&self, output: &SimulationOutput) -> ColumnValue {
        match &self {
            Self::BlocksPublished => {
                let num = output.blockchain.num_blocks() as f64;

                ColumnValue::BlocksPublished(num)
            }
            Self::Constant(s) => {
                let value = s.call(());

                ColumnValue::Constant(value)
            }
            Self::MinerStrategyName(miner_id) => {
                let name = output.miners[*miner_id - 1].name();

                ColumnValue::MinerStrategyName(name)
            }
            Self::MiningPower(miner_id) => {
                // Safety: power distributions are validated during the build
                // step of the simulation pipeline
                let power = unsafe {
                    output
                        .power_dist
                        .power_of_unchecked(*miner_id, output.miners.len())
                };

                ColumnValue::MiningPower(power)
            }
            Self::MiningPowerFunction(miner_id, func) => {
                // Safety: power distributions are validated during the build
                // step of the simulation pipeline
                let power = unsafe {
                    output
                        .power_dist
                        .power_of_unchecked(*miner_id, output.miners.len())
                };
                let value = func.call(power);

                ColumnValue::MiningPowerFunction(value)
            }
            Self::MinerRevenue(miner_id) => {
                let revenue = revenue_of(miner_id, output);

                ColumnValue::MinerRevenue(revenue)
            }
            Self::Rounds => {
                let rounds = output.rounds;

                ColumnValue::Rounds(rounds)
            }
            Self::LongestChainLength => {
                let length = output.longest_chain.len() as f64;

                ColumnValue::LongestChainLength(length)
            }
            Self::AverageOf => {
                let times = 1;

                ColumnValue::AverageOf(times)
            }
        }
    }

    fn get_average_value(&self, data: &[SimulationOutput]) -> ColumnValue {
        match &self {
            Self::BlocksPublished => {
                let mut num = data
                    .iter()
                    .map(|sim_output| sim_output.blockchain.num_blocks() as f64)
                    .sum();
                num /= data.len() as f64;

                ColumnValue::BlocksPublished(num)
            }
            Self::MinerRevenue(miner_id) => {
                let mut revenue = data
                    .iter()
                    .map(|sim_output| revenue_of(miner_id, sim_output))
                    .sum();
                revenue /= data.len() as f64;

                ColumnValue::MinerRevenue(revenue)
            }
            Self::LongestChainLength => {
                let mut length = data
                    .iter()
                    .map(|sim_output| sim_output.longest_chain.len() as f64)
                    .sum();
                length /= data.len() as f64;

                ColumnValue::LongestChainLength(length)
            }
            Self::AverageOf => {
                let times = data.len();

                ColumnValue::AverageOf(times)
            }
            // otherwise use the first simulation's value
            Self::Constant(_)
            | Self::MinerStrategyName(_)
            | Self::MiningPower(_)
            | Self::MiningPowerFunction(_, _)
            | Self::Rounds => self.get_value(&data[0]),
        }
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::AverageOf => {
                write!(f, "Average Of")
            }
            Self::BlocksPublished => {
                write!(f, "Blocks Published")
            }
            Self::Constant(func) => {
                write!(f, "{}", func.name())
            }
            Self::MinerStrategyName(miner_id) => {
                write!(f, "Miner {} Strategy", miner_id)
            }
            Self::MiningPower(miner_id) => {
                write!(f, "Miner {} Power", miner_id)
            }
            Self::MiningPowerFunction(_, func) => {
                write!(f, "{}", func.name())
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

impl Display for ColumnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::AverageOf(repeats) => {
                write!(f, "{}", repeats)
            }
            Self::BlocksPublished(num) => {
                write!(f, "{:.1$}", num, FLOAT_PRECISION_DIGITS)
            }
            Self::Constant(value) => {
                write!(f, "{:.1$}", value, FLOAT_PRECISION_DIGITS)
            }
            Self::MinerStrategyName(name) => {
                write!(f, "{}", name)
            }
            Self::MiningPower(power) => {
                write!(f, "{:.1$}", power, FLOAT_PRECISION_DIGITS)
            }
            Self::MiningPowerFunction(value) => {
                write!(f, "{:.1$}", value, FLOAT_PRECISION_DIGITS)
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
