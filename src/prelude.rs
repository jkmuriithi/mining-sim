/*!
Re-export of common values and datatypes.

Must be imported manually.

```
use mining_sim::prelude::*;
```
*/

use crate::{
    blockchain, miner, power_dist, results, simulation, tie_breaker,
    transaction,
};

pub use blockchain::{Block, BlockId, BlockPublishingError, Blockchain};

pub use miner::{
    honest::Honest, honestforking::HonestForking, ndeficit::NDeficit,
    noop::Noop, selfish::Selfish, Action, Miner, MinerId,
};

pub use power_dist::{PowerDistribution, PowerDistributionError, PowerValue};

pub use results::{Format, SimulationResults, SimulationResultsBuilder};

pub use simulation::{
    SimulationBuildError, SimulationBuilder, SimulationError, SimulationGroup,
    SimulationOutput,
};

pub use tie_breaker::TieBreaker;

pub use transaction::Transaction;