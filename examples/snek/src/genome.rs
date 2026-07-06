use rand::{Rng, RngExt};
use rand::distr::Uniform;
use serde::{Deserialize, Serialize};
use gars::Genotype;

/// A small feed-forward neural network with one hidden layer.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Genome {
    /// The neural network weights.
    ///
    /// The weights are flattened: input + hidden + output.
    pub weights: Vec<f64>,
}

impl Genome {
    /// The number of input weights.
    pub const INPUT_SIZE: usize = 11;
    /// The number of hidden weights.
    pub const HIDDEN_SIZE: usize = 16;
    /// The number of output weights.
    pub const OUTPUT_SIZE: usize = 3;
    /// The total number of weights.
    pub const NUM_WEIGHTS: usize = Self::INPUT_SIZE * Self::HIDDEN_SIZE + Self::HIDDEN_SIZE * Self::OUTPUT_SIZE;

    #[must_use]
    pub fn new(weights: Vec<f64>) -> Self {
        debug_assert_eq!(weights.len(), Self::NUM_WEIGHTS);
        Self { weights }
    }

    /// Feed the input vector through the network and return the chose action.
    pub fn forward(&self, inputs: &[f64]) -> Action {
        debug_assert_eq!(inputs.len(), Self::INPUT_SIZE);
        let w = &self.weights;
        let mut hidden = [0.0; Self::HIDDEN_SIZE];
        // input -> hidden
        for h in 0..Self::HIDDEN_SIZE {
            let mut sum = 0.0;
            for i in 0..Self::INPUT_SIZE {
                sum += w[i * Self::HIDDEN_SIZE + h] * inputs[i];
            }
            hidden[h] = sum.tanh();
        }
        // hidden -> output
        let mut outputs = [0.0; Self::OUTPUT_SIZE];
        for o in 0..Self::OUTPUT_SIZE {
            let mut sum = 0.0;
            for h in 0..Self::HIDDEN_SIZE {
                sum += w[Self::INPUT_SIZE * Self::HIDDEN_SIZE + h * Self::OUTPUT_SIZE + o] * hidden[h];
            }
            outputs[o] = sum;
        }
        // argmax
        let best = outputs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;

        match best {
            0 => Action::Straight,
            1 => Action::TurnLeft,
            _ => Action::TurnRight,
        }
    }
}

impl Genotype for Genome {
    fn random(rng: &mut impl Rng) -> Self {
        let dist = Uniform::new(-1.0, 1.0).unwrap();
        let weights = rng
            .sample_iter(dist)
            .take(Self::NUM_WEIGHTS)
            .collect();
        Self::new(weights)
    }
}

/// Actions a snake can take.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Action {
    Straight,
    TurnLeft,
    TurnRight,
}
