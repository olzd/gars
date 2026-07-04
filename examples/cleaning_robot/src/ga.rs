use rand::{Rng, RngExt};
use rand::distr::StandardUniform;
use gars::{Individual, Evaluator, Operator, Selector};
use crate::genome::Genome;
use crate::grid::Grid;
use crate::simulation::{Simulation, SimulationHistory, StepResult};

pub struct RobotEvaluator {
    pub grid_with: usize,
    pub grid_height: usize,
    pub wall_prob: f64,
    pub obj_prob: f64,
    pub initial_energy: f64,
    pub eval_runs: usize,
    pub obj_reward: f64,
    pub energy_reward: f64,
    pub wall_penalty: f64,
    pub pickup_penalty: f64,
    pub idle_penalty: f64,
}

impl RobotEvaluator {
    fn fitness(&self, history: &SimulationHistory) -> f64 {
        let mut score = f64::from(history.collected_obj) * self.obj_reward;
        for step in &history.steps {
            match step.result {
                StepResult::HitWall => score -= self.wall_penalty,
                StepResult::FailedPickup => score -= self.pickup_penalty,
                StepResult::Idled => score -= self.idle_penalty,
                _ => {}
            }
        }

        if history.remaining_energy > 0.0 {
            score += self.energy_reward;
        }
        score
    }
}

impl Evaluator<Genome> for RobotEvaluator {
    fn evaluate(&self, genotype: &Genome, rng: &mut impl Rng) -> f64 {
        // an individual (its genome) will be evaluated over multiple simulations on randomly
        // generated grids
        let mut total = 0.0;
        for _ in 0..self.eval_runs {
            let grid = Grid::new_random(self.grid_with, self.grid_height, self.obj_prob, self.wall_prob, rng);
            let sim = Simulation::new(grid, self.initial_energy);
            let history = sim.run(genotype, rng);
            let fitness = self.fitness(&history);
            total += fitness;
        }
        total / self.eval_runs as f64
    }
}

pub struct GenomeOperator {
    pub mutation_rate: f64,
}

impl Operator<Genome> for GenomeOperator {
    fn crossover(&self, a: &Genome, b: &Genome, rng: &mut impl Rng) -> (Genome, Genome) {
        // split the parents' genome at a random index
        let m = rng.random_range(1..Genome::SIZE);
        // construct the children's genome by taking half of each of their parents'
        let mut c1 = a.clone();
        let mut c2 = b.clone();
        c1[m..].copy_from_slice(&b[m..]);
        c2[m..].copy_from_slice(&a[m..]);
        (c1, c2)
    }

    fn mutate(&self, genotype: &mut Genome, rng: &mut impl Rng) {
        for gene in genotype.iter_mut() {
            let p: f64 = rng.sample(StandardUniform);
            if p < self.mutation_rate {
                *gene = rng.sample(StandardUniform);
            }
        }
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
