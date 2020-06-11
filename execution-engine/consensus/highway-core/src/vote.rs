use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::{
    state::State,
    traits::{Context, ValidatorSecret},
    validators::ValidatorIndex,
    vertex::SignedWireVote,
};

/// The observed behavior of a validator at some point in time.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::Hash: Serialize",
    deserialize = "C::Hash: Deserialize<'de>",
))]
pub enum Observation<C: Context> {
    /// No vote by that validator was observed yet.
    None,
    /// The validator's latest vote.
    Correct(C::Hash),
    /// The validator has been seen
    Faulty,
}

impl<C: Context> Observation<C> {
    /// Returns the vote hash, if this is a correct observation.
    pub fn correct(&self) -> Option<&C::Hash> {
        match self {
            Self::None | Self::Faulty => None,
            Self::Correct(hash) => Some(hash),
        }
    }

    fn is_correct(&self) -> bool {
        match self {
            Self::None | Self::Faulty => false,
            Self::Correct(_) => true,
        }
    }
}

/// The observed behavior of all validators at some point in time.
#[derive(Clone, Debug, Deref, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::Hash: Serialize",
    deserialize = "C::Hash: Deserialize<'de>",
))]
pub struct Panorama<C: Context>(pub Vec<Observation<C>>);

impl<C: Context> Panorama<C> {
    /// Creates a new, empty panorama.
    pub fn new(num_validators: usize) -> Panorama<C> {
        Panorama(vec![Observation::None; num_validators])
    }

    /// Returns the observation for the given validator. Panics if the index is out of range.
    pub fn get(&self, idx: ValidatorIndex) -> &Observation<C> {
        &self.0[idx.0 as usize]
    }

    /// Returns `true` if there is no correct observation yet.
    pub fn is_empty(&self) -> bool {
        !self.iter().any(Observation::is_correct)
    }

    /// Returns an iterator over all observations, by validator index.
    pub fn enumerate(&self) -> impl Iterator<Item = (ValidatorIndex, &Observation<C>)> {
        self.iter()
            .enumerate()
            .map(|(idx, obs)| (ValidatorIndex(idx as u32), obs))
    }

    /// Returns an iterator over all correct latest votes, by validator index.
    pub fn enumerate_correct(&self) -> impl Iterator<Item = (ValidatorIndex, &C::Hash)> {
        self.enumerate()
            .filter_map(|(idx, obs)| obs.correct().map(|vhash| (idx, vhash)))
    }

    /// Updates this panorama by adding one vote. Assumes that all justifications of that vote are
    /// already seen.
    pub fn update(&mut self, idx: ValidatorIndex, obs: Observation<C>) {
        self.0[idx.0 as usize] = obs;
    }
}

/// A vote sent to or received from the network.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote<C: Context> {
    // TODO: Signature
    /// The list of latest messages and faults observed by the sender of this message.
    pub panorama: Panorama<C>,
    /// The number of earlier messages by the same sender.
    pub seq_number: u64,
    /// The validator who created and sent this vote.
    pub sender: ValidatorIndex,
    /// The block this is a vote for. Either it or its parent must be the fork choice.
    pub block: C::Hash,
    /// A skip list index of the sender's swimlane, i.e. the previous vote by the same sender.
    ///
    /// For every `p = 1 << i` that divides `seq_number`, this contains an `i`-th entry pointing to
    /// the older vote with `seq_number - p`.
    pub skip_idx: Vec<C::Hash>,
    /// This vote's instant, in milliseconds since the epoch.
    pub instant: u64,
    /// Original signature of the `SignedWireVote`.
    pub signature: <C::ValidatorSecret as ValidatorSecret>::Signature,
}

impl<C: Context> Vote<C> {
    /// Creates a new `Vote` from the `WireVote`, and returns the values if it contained any.
    /// Values must be stored as a block, with the same hash.
    pub fn new(
        swvote: SignedWireVote<C>,
        fork_choice: Option<&C::Hash>,
        state: &State<C>,
    ) -> (Vote<C>, Option<Vec<C::ConsensusValue>>) {
        let block = if swvote.wire_vote.values.is_some() {
            swvote.wire_vote.hash() // A vote with a new block votes for itself.
        } else {
            // If the vote didn't introduce a new block, it votes for the fork choice itself.
            // `Highway::add_vote` checks that the panorama is not empty.
            fork_choice
                .cloned()
                .expect("nonempty panorama has nonempty fork choice")
        };
        let mut skip_idx = Vec::new();
        if let Some(hash) = swvote
            .wire_vote
            .panorama
            .get(swvote.wire_vote.sender)
            .correct()
        {
            skip_idx.push(hash.clone());
            for i in 0..swvote.wire_vote.seq_number.trailing_zeros() as usize {
                let old_vote = state.vote(&skip_idx[i]);
                skip_idx.push(old_vote.skip_idx[i].clone());
            }
        }
        let vote = Vote {
            panorama: swvote.wire_vote.panorama,
            seq_number: swvote.wire_vote.seq_number,
            sender: swvote.wire_vote.sender,
            block,
            skip_idx,
            instant: swvote.wire_vote.instant,
            signature: swvote.signature,
        };
        (vote, swvote.wire_vote.values)
    }

    /// Returns the sender's previous message.
    pub fn previous(&self) -> Option<&C::Hash> {
        self.skip_idx.first()
    }
}
