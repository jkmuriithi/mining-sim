/*!
A library for simulating strategic blockchain mining outcomes based on
game-theoretical models.

## Todo:
- Miner names (String field, &str getter, default "Miner X")
- SM, NSM, N-Deficit Strategy Defs
- Multi-threading in simulation runs, corresponding options in SimulationBuilder

## Issues:
*/

pub mod block;
pub mod blockchain;
pub mod miner;
pub mod simulation;
pub mod transaction;

pub use simulation::create;
