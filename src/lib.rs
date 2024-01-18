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

- Split Simulation into two structs (Simulation and SimulationSet)
- Revise results - move CSV output to a function and make Display "pretty-print"
a table
## Issues:

## Important Tests (Conversation with Weinberg):
- Honest miners only -> alpha is roughly equal to revenue
- Selfish Mining revenue -> alpha matches closed form from Iyal paper
- NSM revenue -> alpha matches closed form from Weinberg-Ferreira
*/

pub mod block;
pub mod blockchain;
pub mod miner;
pub mod simulation;
pub mod transaction;

pub use simulation::{SimulationBuilder, SimulationResults};
