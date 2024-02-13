/*!
Control the appearance of simulation result data

# Working with [`ResultsBuilder`]

## Examples

Creating a [`ResultsTable`] after running a simulation group:

```
use mining_sim::prelude::*;

let sim = SimulationBuilder::new()
    .add_miner(Honest::new())
    .add_miner(Honest::new())
    .repeat_all(5)
    .power_values([0.1, 0.9])
    .build()
    .unwrap();

let results_builder = sim.run_all().unwrap();

let results = results_builder
    .average(Average::Median) // Take the median of repeated simulations' results
    .rounds()                 // Include the number of rounds run per simulation
    .format(Format::CSV)      // Output results as CSV
    .build();

println!("{}", results);
```

# Formatting Results


# Aggregating Results
TODO: Specify and implementing different aggregation methods (median, mean, max, min)
for repeated sims

*/

use std::{collections::BTreeSet, fmt::Display, num::NonZeroUsize};

use rayon::prelude::*;

use crate::{
    miner::MinerId, power_dist::PowerValue, simulation::SimulationOutput,
    utils::wrap, utils::WrapFunc,
};

/// Floating point precision of results data.
pub const FLOAT_PRECISION_DIGITS: usize = 6;

/// Builder for [`ResultsTable`]. Typically produced by running a
/// [`SimulationGroup`](crate::simulation::SimulationGroup).
#[derive(Debug, Clone)]
pub struct ResultsBuilder {
    average: Average,
    columns: BTreeSet<Column>,
    data: Vec<SimulationOutput>,
    format: Format,
    repeated: NonZeroUsize,
}

/// Describes the appearance of a [`ResultsTable`] table as given by its
/// [`Display`] implementation.
#[derive(Debug, Clone, Copy, Default)]
pub enum Format {
    /// Comma-separated, without extra whitespace.
    CSV,
    /// Human-readable.
    #[default]
    PrettyPrint,
}

impl ResultsBuilder {
    /// Create a new [`ResultsBuilder`].
    pub(crate) fn new(
        data: Vec<SimulationOutput>,
        repeated: NonZeroUsize,
    ) -> Self {
        Self {
            data,
            repeated,
            average: Average::default(),
            columns: BTreeSet::default(),
            format: Format::default(),
        }
    }

    /// Include the "Blocks Published", "Longest Chain Length",
    /// "Miner `X` Strategy Name", "Miner `X` Revenue", and "Simulated Rounds"
    /// columns.
    ///
    /// [`ResultsBuilder::average`] must still be called separately
    /// to create averaged data.
    pub fn all(self) -> Self {
        self.blocks_published()
            .longest_chain_length()
            .strategy_names()
            .revenue()
            .rounds()
    }

    /// Average the results of repeated simulations based on the given
    /// [`Average`] type. For types other than [`Average::None`], a column
    /// describing the averaging method will be included in the results table.
    pub fn average(mut self, average: Average) -> Self {
        self.average = average;

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
        self.columns.insert(Column::Constant(wrap!(title, move |_| value)));

        self
    }

    /// Extract the raw [`SimulationOutput`] data from this [`ResultsBuilder`].
    /// Useful for running custom statistical analysis.
    ///
    /// # Ordering
    /// Simulations are run in the same order they are specified using
    /// [`SimulationBuilder`], with repeated runs being grouped together.
    /// The output data from this method follows this ordering as well.
    pub fn data(self) -> Vec<SimulationOutput> {
        self.data
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
        miner_id: MinerId,
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
            self.columns.insert(Column::MinerStrategyName(miner_id.into()));
        }

        self
    }

    /// Include a "Miner `X` Revenue" column in the results table for each
    /// miner `X`.
    pub fn revenue(mut self) -> Self {
        let num_miners = self.data[0].miners.len();
        for miner_id in 1..=num_miners {
            self.columns.insert(Column::MinerRevenue(miner_id.into()));
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

    /// Create new [`ResultsTable`].
    pub fn build(self) -> ResultsTable {
        let ResultsBuilder { average, mut columns, data, format, repeated } =
            self;

        let num_miners = data[0].miners.len();
        for miner_id in 1..=num_miners {
            columns.insert(Column::MiningPower(miner_id.into()));
        }

        match average {
            Average::None => (),
            _ => {
                columns.insert(Column::AverageOf(average));
            }
        }

        let columns = Vec::from_iter(columns);
        let rows = match average {
            Average::None => data
                .iter()
                .map(|sim_output| {
                    columns
                        .par_iter()
                        .map(|col_type| col_type.get_value(sim_output))
                        .collect()
                })
                .collect(),
            _ => data
                .chunks(repeated.get())
                .map(|sim_outputs| {
                    columns
                        .par_iter()
                        .map(|col_type| {
                            col_type.get_average_value(average, sim_outputs)
                        })
                        .collect()
                })
                .collect(),
        };

        ResultsTable { columns, format, rows }
    }
}

/// Formatted results from the completion of a
/// [`SimulationGroup`](crate::simulation::SimulationGroup). The results table
/// is given by the struct's [`Display`] implementation, as specified by
/// its [`Format`].
pub struct ResultsTable {
    columns: Vec<Column>,
    format: Format,
    rows: Vec<Vec<ColumnValue>>,
}

impl ResultsTable {
    const SEPARATOR_VERTICAL: char = '|';
    const SEPARATOR_HORIZONTAL: char = '-';

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn set_format(&mut self, format: Format) {
        self.format = format;
    }
}

impl Display for ResultsTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let titles: Vec<_> =
            self.columns.iter().map(|col_type| col_type.to_string()).collect();

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

/// Methods of extracting an average/central value from a set of repeated
/// simulations.
///
/// In the process of creating an results table, the given averaging method is
/// only applied to the values of columns which change over time.
#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Average {
    #[default]
    /// Include all repeated values.
    None,
    /// Arithmetic mean of all values.
    Mean,
    /// Median of all values.
    Median,
    /// Maximum of all values.
    Max,
    /// Minimum of all values.
    Min,
}

/// Type of column that can appear in a data table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Column {
    // Variant order determines the order of columns in results tables:
    // https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html#derivable
    MinerStrategyName(MinerId),
    MiningPower(MinerId),
    MinerRevenue(MinerId),
    MiningPowerFunction(MinerId, WrapFunc<PowerValue, f64>),
    Constant(WrapFunc<(), f64>),
    Rounds,
    AverageOf(Average),
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
fn revenue_of(miner_id: &MinerId, data: &SimulationOutput) -> f64 {
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
                let name = output.miners[miner_id.0 - 1].name();

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
            Self::AverageOf(_) => unreachable!(
                "never need the single value of the average descriptor column"
            ),
        }
    }

    fn get_average_value(
        &self,
        method: Average,
        data: &[SimulationOutput],
    ) -> ColumnValue {
        match &self {
            Self::AverageOf(_) => return ColumnValue::AverageOf(data.len()),
            Self::Constant(_)
            | Self::MinerStrategyName(_)
            | Self::MiningPower(_)
            | Self::MiningPowerFunction(_, _)
            | Self::Rounds => return self.get_value(&data[0]),
            Self::BlocksPublished => (),
            Self::MinerRevenue(_) => (),
            Self::LongestChainLength => (),
        }

        let vls: Vec<_> = match &self {
            Self::BlocksPublished => data
                .iter()
                .map(|sim_output| sim_output.blockchain.num_blocks() as f64)
                .collect(),
            Self::MinerRevenue(miner_id) => data
                .iter()
                .map(|sim_output| revenue_of(miner_id, sim_output))
                .collect(),
            Self::LongestChainLength => data
                .iter()
                .map(|sim_output| sim_output.longest_chain.len() as f64)
                .collect(),
            _ => unreachable!(),
        };

        let avg = match method {
            Average::Mean => vls.into_iter().sum::<f64>() / data.len() as f64,
            Average::Median => crate::utils::median_of_floats(vls),
            Average::Max => vls.into_iter().reduce(|a, b| a.max(b)).unwrap(),
            Average::Min => vls.into_iter().reduce(|a, b| a.min(b)).unwrap(),
            Average::None => unreachable!(),
        };

        match &self {
            Self::BlocksPublished => ColumnValue::BlocksPublished(avg),
            Self::MinerRevenue(_) => ColumnValue::MinerRevenue(avg),
            Self::LongestChainLength => ColumnValue::LongestChainLength(avg),
            _ => unreachable!(),
        }
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::AverageOf(method) => match method {
                Average::Mean => write!(f, "Mean Of"),
                Average::Median => write!(f, "Median Of"),
                Average::Max => write!(f, "Max Of"),
                Average::Min => write!(f, "Min Of"),
                Average::None => unreachable!(),
            },
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

/// Returns an instance of the ideal Selfish Miner revenue function from Eyal
/// and Sirer's paper which can be used as input to
/// [`ResultsBuilder::mining_power_func`].
pub fn selfish_revenue(gamma: f64) -> impl Fn(PowerValue) -> f64 {
    move |a: PowerValue| -> f64 {
        (a * (1.0 - a).powi(2) * (4.0 * a + gamma * (1.0 - 2.0 * a))
            - a.powi(3))
            / (1.0 - a * (1.0 + a * (2.0 - a)))
    }
}

/// Ideal Nothing-At-Stake miner revenue function from Weinberg and Ferrera's
/// paper. Can be used as input to
/// [`ResultsBuilder::mining_power_func`].
pub fn nsm_revenue(a: PowerValue) -> f64 {
    (4.0 * a.powi(2) - 8.0 * a.powi(3) - a.powi(4) + 7.0 * a.powi(5)
        - 3.0 * a.powi(6))
        / (1.0 - a - 2.0 * a.powi(2) + 3.0 * a.powi(4) - 3.0 * a.powi(5)
            + a.powi(6))
}
