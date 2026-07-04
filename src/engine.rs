use rand::rngs::ChaCha8Rng;
use rand::SeedableRng;
use rayon::prelude::*;
use crate::ga::{Evaluator, Genotype, Individual, Operator, Selector};
use crate::observer::{GenerationObserver, GenerationStats};

/// The main genetic algorithm engine.
/// 
/// The engine manages the entire evolution loop: evaluation, selection, crossover and mutation.
/// It is fully generic over:
/// 
/// * `G` - the [`genotype`][`Genotype`],
/// * `E` - the [`evaluator`][`Evaluator`],
/// * `O` - the genetic [`operators`][`Operator`],
/// * `S` - the selection operator.
/// 
/// # Determinism and parallelism
/// 
/// The engine is deterministic given a master seed. Parallel evaluation uses `ChaCha8Rng` with
/// [`ChaCha8Rng::set_stream`] to assign a unique, reproducible RNG to each individual.
/// 
/// # Observers
/// 
/// Multiple [`GenerationObserver`] can be registered via [`Engine::add_observer`] and they will
/// be notified in order after every generation evaluation.
/// 
/// # History
/// 
/// Per-generation statistics (see [`GenerationStats`]) are automatically recorded.
pub struct Engine<G: Genotype, E: Evaluator<G>, O: Operator<G>, S: Selector<G>> {
    config: Config,
    evaluator: E,
    operator: O,
    selector: S,
    population: Vec<Individual<G>>,
    generation: usize,
    best: Option<Individual<G>>,
    evolution_rng: ChaCha8Rng,
    history: Vec<GenerationStats>,
    observers: Vec<Box<dyn GenerationObserver<G>>>,
}

impl<G: Genotype, E: Evaluator<G>, O: Operator<G>, S: Selector<G>> Engine<G, E, O, S> {
    /// Create a new engine with an initial random population.
    /// 
    /// # Parameters
    /// 
    /// * `config` - the engine general [`configuration`][`Config`].
    /// * `evaluator`- the problem-specific fitness function.
    /// * `operator` - the crossover and mutation implementation.
    /// * `selector`- the parent selection strategy for reproduction.
    #[must_use]
    pub fn new(config: Config, evaluator: E, operator: O, selector: S) -> Self {
        // initialize the evolution RNG from the seed
        let evolution_seed = config.seed.wrapping_add(0xdeadbeef);
        let evolution_rng = ChaCha8Rng::seed_from_u64(evolution_seed);
        // initialize the population with a temporary seeded RNG
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let population = (0..config.population_size)
            .map(|_| Individual::new_random(&mut rng))
            .collect();

        Self {
            config,
            evaluator,
            operator,
            selector,
            population,
            generation: 0,
            best: None,
            evolution_rng,
            history: Vec::new(),
            observers: Vec::new(),
        }
    }

    /// Return the complete generation statistics. 
    pub fn history(&self) -> &[GenerationStats] {
        &self.history
    }

    /// Return the best individual found so far (`None` if no generation has been evaluated yet).
    pub fn best(&self) -> Option<&Individual<G>> {
        self.best.as_ref()
    }

    /// Return the current generation number (0-based).
    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Register a new [`observer`][`GenerationObserver`] that will be notified after every
    /// generation evaluation.
    /// 
    /// Observers will be notified in the order they were added.
    pub fn add_observer(&mut self, observer: Box<dyn GenerationObserver<G>>) {
        self.observers.push(observer);
    }

    /// Advance one generation:
    /// 1. evaluate current population
    /// 2. compute stats
    /// 4. notify registered observers
    /// 3. evolve population
    /// 
    /// This is the heart of the engine. Call it manually to step through, or use [`Engine::run`]
    /// to iterate automatically.
    pub fn step(&mut self) {
        // evaluate population
        self.evaluate();

        // compute stats
        let best = self.population
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap();

        let avg_fitness = self.population
            .iter()
            .map(|p| p.fitness)
            .sum::<f64>() / self.population.len() as f64;

        let stats = GenerationStats {
            generation: self.generation,
            best_fitness: best.fitness,
            avg_fitness,
        };

        // notify observers
        for obs in &mut self.observers {
            obs.on_generation(&stats, self.best.as_ref().unwrap());
        }

        self.history.push(stats);
        self.best = Some(best.clone());

        // evolve the population
        if self.generation < self.config.max_generations {
            self.evolve();
        }
        self.generation += 1;
    }

    /// Run all remaining generations.
    /// 
    /// This is a convenient wrapper around [`Engine::step`].
    pub fn run(&mut self) {
        while self.generation < self.config.max_generations {
            self.step();
        }
    }

    /// Evaluate every individual in the current population.
    ///
    /// Note that each individual will be evaluated in parallel and will use their own RNG seeded
    /// from [`Config::seed`]. To instead use a common RNG to evaluate the individuals, ignore the
    /// provided one and implement the logic in the [evaluator][`Evaluator`] implementation.
    fn evaluate(&mut self) {
        self.population
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, individual)| {
                let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed);
                rng.set_stream(i as u64);
                individual.fitness = self.evaluator.evaluate(&individual.genotype, &mut rng);
            });
    }

    /// Evolve the current population to produce a new generation.
    ///
    /// Start by keeping [`Config::elite_count`] best individuals, then fill the population until
    /// [`Config::population_size`] by repeating the following steps:
    ///
    /// 1. select (see [`Selector::select`])
    /// 2. crossover (see [`Operator::crossover`])
    /// 3. mutate (see [`Operator::mutate`])
    /// 4. add the new individuals to the new generation
    fn evolve(&mut self) {
        // create new population preserving the elites
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        let mut population = Vec::with_capacity(self.config.population_size);
        for i in 0..self.config.elite_count {
            population.push(self.population[i].clone());
        }

        while population.len() < self.config.population_size {
            // select 2 parents
            let p1 = self.selector.select(&self.population, &mut self.evolution_rng);
            let p2 = self.selector.select(&self.population, &mut self.evolution_rng);
            // apply crossover
            let (mut g1, mut g2) = self.operator.crossover(&p1.genotype, &p2.genotype, &mut self.evolution_rng);
            // mutate children
            self.operator.mutate(&mut g1, &mut self.evolution_rng);
            self.operator.mutate(&mut g2, &mut self.evolution_rng);
            // add to population
            population.push(Individual::new(g1));
            if population.len() < self.config.population_size {
                population.push(Individual::new(g2));
            }
        }
        self.population = population;
    }
}

/// The [`Engine`] configuration.
/// 
/// It does not hold parameters for problem-specific components ([`evaluator`][`Evaluator`],
/// [`operators`][`Operator`] or [`selector`][`Selector`]) as they are the concrete implementations'
/// responsibility.
pub struct Config {
    /// The population size, must be greater than `elite_count`.
    pub population_size: usize,
    /// The maximum number of generations to simulate.
    pub max_generations: usize,
    /// The number of best individuals to keep in the next generation.
    pub elite_count: usize,
    /// The seed to use for the RNGs for full reproducibility.
    pub seed: u64,
}
