use rand::Rng;

/// The representation of a candidate solution.
///
/// This is a [genetic representation](https://en.wikipedia.org/wiki/Genetic_representation) of the
/// solution domain.
///
/// # Requirements
///
/// * `Clone` - genotypes must be duplicable.
/// * `Send + Sync` - genotypes are evaluated in parallel by the engine.
/// * `'static` - no borrowed data, the engine owns the population.
pub trait Genotype: Clone + Send + Sync + 'static {
    /// Construct a new random genotype.
    ///
    /// The provided `rng` is deterministic and seeded by the [`engine`][`crate::Engine`] to guarantee a
    /// reproducible initial population generation.
    fn random(rng: &mut impl Rng) -> Self;
}

/// The [fitness function](https://en.wikipedia.org/wiki/Fitness_function).
///
/// The evaluator is the **only** problem-specific code the [`engine`][`crate::Engine`] calls.
/// It must be deterministic with respect to the supplied RNG: given the same genotype and RNG
/// state, the same fitness score must be returned.
///
/// The engine provides a unique, reproducible RNG per individual to guarantee that evaluations are
/// parallel-safe yet deterministic.
///
/// # Requirements
///
/// * `Sync` - the engine evaluates the population in parallel and will share the evaluator.
pub trait Evaluator<G: Genotype>: Sync {
    /// Compute the fitness of a single genotype.
    ///
    /// # Parameters
    ///
    /// * `genotype`- the individual to evaluate.
    /// * `rng` - a deterministic random number generator specifically seeded for this individual.
    /// **Do not re-seed it**.
    fn evaluate(&self, genotype: &G, rng: &mut impl Rng) -> f64;
}

/// The genetic operators: crossover and mutation.
///
/// They are sequentially applied by the engine to create the next generation. The engine guarantee
/// the operators are called with a deterministic RNG that advances across generations.
pub trait Operator<G: Genotype> {
    /// Combine two parent genotypes to produce two children.
    fn crossover(&self, a: &G, b: &G, rng: &mut impl Rng) -> (G, G);

    /// Mutate a genotype by applying random perturbations.
    fn mutate(&self, genotype: &mut G, rng: &mut impl Rng);
}

/// The selection operator.
///
/// Pick an individual from the current population to reproduce for the next generation. The engine
/// guarantee the selector is called with a deterministic RNG.
pub trait Selector<G: Genotype> {
    /// Select an individual from the population.
    ///
    /// # Parameters
    ///
    /// * `population` - the current generation's evaluated individuals.
    /// * `rng` - a deterministic RNG for this generation's evolution phase.
    ///
    /// # Returns
    ///
    /// A reference to an individual inside the current `population`.
    fn select<'a>(&self, population: &'a [Individual<G>], rng: &mut impl Rng) -> &'a Individual<G>;
}

/// Wrapper around a [`genotype`][`Genotype`] and its fitness.
#[derive(Clone)]
pub struct Individual<G: Genotype> {
    pub genotype: G,
    /// The last computed fitness of this individual's genotype.
    ///
    /// This value is updated and used by the engine's [`step`][`crate::Engine::step`].
    pub fitness: f64,
}

impl<G: Genotype> Individual<G> {
    /// Construct a new individual from the given [`genotype`][`Genotype`].
    /// Its [`fitness`][`Individual::fitness`] will be initialized to 0.
    pub fn new(genotype: G) -> Self {
        Self {
            genotype,
            fitness: 0.0,
        }
    }

    /// Construct a new individual with a random [`genotype`][`Genotype`].
    /// Its [`fitness`][`Individual::fitness`] will be initialized to 0.
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let genotype = G::random(rng);
        Self::new(genotype)
    }
}
