//! Describing tie-breaking behavior in miner strategies

use rand::{seq::SliceRandom, Rng};

use crate::{block::BlockID, blockchain::Blockchain, miner::MinerID};

/// Breaks ties between multiple blocks of at the tip of a blockchain's longest
/// chain.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TieBreaker {
    /// Use the block published in the earliest round.
    #[default]
    EarliestPublished,
    /// Use the earliest block published by the specified miner, if such a block
    /// exists. Otherwise, use the earliest block published by any miner.
    FavorMiner(MinerID),
    /// With the given probability, use the earliest block published by the
    /// specified miner, if such a block exists. Otherwise, use the earliest
    /// block published by any *other* miner.
    FavorMinerProb(MinerID, f64),
    /// Use a block picked uniformly at random.
    Random,
}

impl TieBreaker {
    /// Returns the block at the tip of the longest chain in `blockchain`,
    /// according to the given tie-breaking rule.
    pub fn choose(&self, blockchain: &Blockchain) -> BlockID {
        let tip = blockchain.tip();
        let mut rng = rand::thread_rng();

        match &self {
            Self::EarliestPublished => tip[0],
            Self::FavorMiner(miner_id) => {
                let block_id = tip
                    .iter()
                    .find(|&block_id| {
                        blockchain[block_id].block.miner_id.eq(miner_id)
                    })
                    .copied();

                match block_id {
                    Some(block_id) => block_id,
                    None => tip[0],
                }
            }
            Self::FavorMinerProb(miner_id, prob) => {
                assert!(
                    (0.0..=1.0).contains(prob),
                    "tie breaker probability must be between 0 and 1"
                );

                let favored = tip
                    .iter()
                    .find(|&block_id| {
                        blockchain[block_id].block.miner_id.eq(miner_id)
                    })
                    .copied();
                let not_favored = tip
                    .iter()
                    .find(|&block_id| {
                        blockchain[block_id].block.miner_id.ne(miner_id)
                    })
                    .copied();

                match (favored, not_favored) {
                    (Some(block_id), None) | (None, Some(block_id)) => block_id,
                    (Some(favored), Some(not_favored)) => {
                        if rng.gen_bool(*prob) {
                            favored
                        } else {
                            not_favored
                        }
                    }
                    (None, None) => {
                        unreachable!("blockchain tip cannot be empty")
                    }
                }
            }
            Self::Random => *tip.choose(&mut rng).unwrap(),
        }
    }
}
