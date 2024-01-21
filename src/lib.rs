/*!
A library for simulating strategic blockchain mining outcomes based on
game-theoretical models.

## Todo:
- ~~Miner names (String field, &str getter, default "Miner X")~~
- ~~SimulationBuilder::miner_alpha_range~~
- ~~SM~~, NSM, N-Deficit Strategy Defs
- Overhaul AlphaDist into a simulation::Distribution (methods values() and
  validate())
- Multi-threading in simulation runs, corresponding options in SimulationBuilder
- Panic descriptions for builder methods
- Variable renaming for clarity
- Honest forking strategy (pretends it didn't hear certain blocks) and/or
  global simulation latency modeling
    - one latency model: exponential distribution of blocks which
    - forking introduces the question of chain throughput (easy metric to implement)
- Handling all selfish miners:
    - A strategy is "excessively patient" if it never publishes the first
      block
    - Can we tell when a strategy is "excessively patient"?
      (Or use a flag to let users know when that's a problem)
- Add a real Genesis miner (implement and use a simple Miner that always waits)
- Optimize ResultsBuilder/Results:
  - over a 50 mil round execution:
    - PowerDist -> Simulation took 0.0007 secs
    - Simulation running took 8.2143 secs
    - Building and printing results took **20.6052 secs**


## Issues:

## Important Tests (Conversation with Weinberg):
- Honest miners only -> alpha is roughly equal to revenue
- Selfish Mining revenue -> alpha matches closed form from Iyal paper
- NSM revenue -> alpha matches closed form from Weinberg-Ferreira
*/

pub mod block;
pub mod blockchain;
pub mod miner;
pub mod power_dist;
pub mod simulation;
pub mod transaction;

pub use blockchain::{BlockPublishingError, Blockchain};

pub use miner::Miner;

pub use power_dist::{PowerDistribution, PowerDistributionError, PowerValue};

pub use simulation::{
    SimulationBuildError, SimulationBuilder, SimulationGroup, SimulationOutput,
    SimulationResults, SimulationResultsBuilder,
};
