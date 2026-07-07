use crate::ga::{Genotype, Individual};

/// An observer that receives a callback after every generation evaluation by the
/// [`engine`][`crate::Engine`].
///
/// Observers are registered with [`crate::Engine::add_observer`] and called sequentially on the main
/// thread. They can be used for logging, visualization, ...
pub trait GenerationObserver<G: Genotype> {
    /// The engine will call this function once per generation, after evaluation.
    ///
    /// # Parameters
    ///
    /// * `stats`- the fitness statistics for the evaluated generation.
    /// * `best`- the individual with the highest fitness in the generation.
    fn on_generation(&mut self, stats: &GenerationStats, best: &Individual<G>);
}

/// Summary statistics of a single generation, recorded by the [`engine`][`crate::Engine`].
///
/// It is passed to every registered [`GenerationObserver`].
#[derive(Debug, Clone)]
pub struct GenerationStats {
    /// The generation number (0-based).
    pub generation: usize,
    /// The highest fitness value found in the population.
    pub best_fitness: f64,
    /// The average fitness of the population.
    pub avg_fitness: f64,
}

impl GenerationStats {
    #[must_use]
    pub fn new(generation: usize, best_fitness: f64, avg_fitness: f64) -> Self {
        Self {
            generation,
            best_fitness,
            avg_fitness,
        }
    }
}
