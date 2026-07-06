use rand::{Rng, RngExt};
use gars::{Evaluator, Individual, Operator, Selector};
use crate::genome::Genome;
use crate::simulation::Simulation;

pub struct SnakeEvaluator {
    pub grid_width: usize,
    pub grid_height: usize,
    pub max_steps: usize,
    pub eval_runs: usize,
}

impl Evaluator<Genome> for SnakeEvaluator {
    fn evaluate(&self, genotype: &Genome, rng: &mut impl Rng) -> f64 {
        let mut total = 0.0;
        for _ in 0..self.eval_runs {
            let mut sim = Simulation::new(self.grid_width, self.grid_height, self.max_steps, rng);
            let snake_init_len = sim.snake.len();
            sim.run(genotype, rng);
            let eaten = sim.snake.len() - snake_init_len;
            total += eaten as f64;
        }
        total / self.eval_runs as f64
    }
}

pub struct GenomeOperator {
    pub mutation_rate: f64,
    pub mutation_strength: f64,
}

impl Operator<Genome> for GenomeOperator {
    fn crossover(&self, a: &Genome, b: &Genome, rng: &mut impl Rng) -> (Genome, Genome) {
        // split the parents' genome at a random index
        let m = rng.random_range(1..Genome::NUM_WEIGHTS);
        // construct the children's genome by taking half of each of their parents'
        let mut c1 = a.weights.clone();
        let mut c2 = b.weights.clone();
        c1[m..].copy_from_slice(&b.weights[m..]);
        c2[m..].copy_from_slice(&a.weights[m..]);
        (Genome::new(c1), Genome::new(c2))
    }

    fn mutate(&self, genotype: &mut Genome, rng: &mut impl Rng) {
        let mut new_weights = genotype.weights.clone();
        for w in new_weights.iter_mut() {
            if rng.random::<f64>() < self.mutation_rate {
                *w += rng.random_range(-self.mutation_strength..self.mutation_strength);
                *w = w.clamp(-1.0, 1.0);
            }
        }
        genotype.weights = new_weights;
    }
}

pub struct TournamentSelector {
    pub size: usize,
}

impl Selector<Genome> for TournamentSelector {
    fn select<'a>(&self, population: &'a [Individual<Genome>], rng: &mut impl Rng) -> &'a Individual<Genome> {
        (0..self.size)
            .map(|_| &population[rng.random_range(0..population.len())])
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap()
    }
}