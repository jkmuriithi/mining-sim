//! Utilities for describing tie-breaking behavior in strategies.

use rand::Rng;

use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

/// Determines how a [Miner](super::Miner) implementation breaks ties
/// between blocks of height [Blockchain::tip_height].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TieBreaker {
    /// Always uses the earliest published block (the first element of
    /// [Blockchain::tips]).
    #[default]
    EarliestPublished,
    /// Uses the earliest published block by the miner with the given ID. If no
    /// such block exists in [Blockchain::tips], the earliest published block is
    /// returned.
    FavorMiner(MinerID),
    /// Uses the earliest published block by the miner with the given ID in the
    /// longest chain, searching for blocks at levels down to the given depth
    /// back from [Blockchain::max_height] If no such block exists, the earliest
    /// published tip block is used.
    FavorMinerFork(MinerID, usize),
    /// Uses the earliest block published by the miner with the given ID, with
    /// the given probability. Otherwise, the earliest published block is
    /// returned.
    FavorMinerProb(MinerID, f64),
}

impl TieBreaker {
    /// Returns the block at the tip of this blockchain's longest chain
    /// in accordance with the tie breaking strategy.
    pub fn choose(&self, chain: &Blockchain) -> BlockID {
        let tip = chain.tip();
        match &self {
            Self::EarliestPublished => tip[0],
            Self::FavorMiner(miner) => tip
                .iter()
                .find(|&b| chain[b].block.miner_id == *miner)
                .copied()
                .unwrap_or(tip[0]),
            Self::FavorMinerFork(miner, depth) => {
                let lowest = chain.max_height.saturating_sub(*depth);
                for i in (lowest..=chain.max_height).rev() {
                    let curr = chain
                        .at_height(i)
                        .iter()
                        .find(|&b| chain[b].block.miner_id == *miner)
                        .copied();

                    if let Some(id) = curr {
                        return id;
                    }
                }

                tip[0]
            }
            Self::FavorMinerProb(miner, prob) => {
                let favored = tip
                    .iter()
                    .find(|&&b| chain[b].block.miner_id == *miner)
                    .copied();
                let not_favored = tip
                    .iter()
                    .find(|&&b| chain[b].block.miner_id != *miner)
                    .copied();

                match favored {
                    None => not_favored.unwrap(),
                    Some(id) => {
                        if rand::thread_rng().gen_range(0.0..1.0) < *prob {
                            id
                        } else {
                            not_favored.unwrap_or(id)
                        }
                    }
                }
            }
        }
    }
}
