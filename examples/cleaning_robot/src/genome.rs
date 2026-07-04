use std::ops::{Deref, DerefMut};
use rand::distr::{Distribution, StandardUniform};
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use gars::Genotype;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Genome(Vec<Action>);

impl Genome {
    /// The number of genes.
    ///
    /// Encode the [action][`Action`] to take given the state of the grid [tiles][`Tiles`] at the
    /// current position, as well as those of its 4 direct neighbors.<br>
    /// Since the tiles can be of one of 3 states and there are 5 of them to consider there are
    /// a total of 3<sup>5</sup> = 243 possible configurations.
    pub const SIZE: usize = 243;
}

impl Genotype for Genome {
    fn random(rng: &mut impl Rng) -> Self {
        Self(rng.sample_iter(StandardUniform).take(Self::SIZE).collect())
    }
}

impl Deref for Genome {
    type Target = Vec<Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Genome {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Action {
    MoveNorth = 0,
    MoveSouth = 1,
    MoveEast = 2,
    MoveWest = 3,
    Idle = 4,
    Pickup = 5,
    #[default]
    MoveRandom = 6,
}

impl Action {
    /// Assign an energy cost to each action.
    #[must_use]
    pub fn energy_cost(&self) -> f64 {
        match self {
            Self::Idle => 0.5,
            Self::MoveNorth
            | Self::MoveSouth
            | Self::MoveEast
            | Self::MoveWest
            | Self::MoveRandom
            | Self::Pickup => 1.0,
        }
    }
}

impl Distribution<Action> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        let idx = rng.random_range(0..=6);
        match idx {
            0 => Action::MoveNorth,
            1 => Action::MoveSouth,
            2 => Action::MoveEast,
            3 => Action::MoveWest,
            4 => Action::Idle,
            5 => Action::Pickup,
            6 => Action::MoveRandom,
            _ => unreachable!(),
        }
    }
}
