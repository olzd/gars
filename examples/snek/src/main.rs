use crate::ga::{GenomeOperator, SnakeEvaluator, TournamentSelector};
use crate::genome::Genome;
use crate::simulation::Simulation;
use crate::tui::run_watch;
use clap::{Parser, Subcommand};
use gars::{Config, Engine};
use rand::distr::StandardUniform;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};
use std::path::PathBuf;

mod ga;
mod genome;
mod simulation;
mod tui;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the genetic algorithm.
    Run {
        /// The width of the grid
        #[arg(long, default_value = "20")]
        width: usize,
        /// The height of the grid
        #[arg(long, default_value = "20")]
        height: usize,
        /// The maximum of simulation steps allowed
        #[arg(long, default_value = "500")]
        max_steps: usize,
        /// The number of simulations to run to evaluate a snake
        #[arg(long, default_value = "5")]
        eval_runs: usize,
        /// The snake population size
        #[arg(long, default_value = "200")]
        pop_size: usize,
        /// The number of generations to evolve
        #[arg(long, default_value = "1000")]
        generations: usize,
        /// The tournament size for selection
        #[arg(long, default_value = "5")]
        tournament: usize,
        /// The mutation rate
        #[arg(long, default_value = "0.05")]
        mutation_rate: f64,
        /// The mutation strength
        #[arg(long, default_value = "0.2")]
        mutation_strength: f64,
        /// The number of elite individual to keep for the next generation (must be less than the population size)
        #[arg(long, default_value = "10")]
        elite: usize,
        /// The seed for the RNG for reproducible executions (optional)
        #[arg(long)]
        seed: Option<u64>,
        /// The file full path to save the best snake
        #[arg(long, default_value = "best.json")]
        output: PathBuf,
    },
    /// Watch a trained genome play.
    Watch {
        /// The file full path of a saved snake to watch
        #[arg(long)]
        genome: PathBuf,
        /// The width of the grid
        #[arg(long, default_value = "20")]
        width: usize,
        /// The height of the grid
        #[arg(long, default_value = "20")]
        height: usize,
        /// The maximum of simulation steps allowed
        #[arg(long, default_value = "500")]
        max_steps: usize,
        /// The seed for the RNG for reproducible executions (optional)
        #[arg(long)]
        seed: Option<u64>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            width,
            height,
            max_steps,
            eval_runs,
            pop_size,
            generations,
            tournament,
            mutation_rate,
            mutation_strength,
            elite,
            seed,
            output,
        } => {
            let seed = seed.unwrap_or_else(|| rand::rng().sample(StandardUniform));

            let config = Config {
                population_size: pop_size,
                max_generations: generations,
                elite_count: elite,
                seed,
            };

            let evaluator = SnakeEvaluator {
                grid_width: width,
                grid_height: height,
                max_steps,
                eval_runs,
            };

            let operator = GenomeOperator {
                mutation_rate,
                mutation_strength,
            };

            let selector = TournamentSelector { size: tournament };

            let mut engine = Engine::new(config, evaluator, operator, selector);
            engine.run();

            println!("best fitness = {}", engine.best().unwrap().fitness);

            let json = serde_json::to_string_pretty(&engine.best().unwrap().genotype)?;
            std::fs::write(output, json)?;
        }
        Commands::Watch {
            genome,
            width,
            height,
            max_steps,
            seed,
        } => {
            let reader = std::fs::File::open(genome)?;
            let genome: Genome = serde_json::from_reader(reader)?;
            let seed = seed.unwrap_or_else(|| rand::rng().sample(StandardUniform));
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            let sim = Simulation::new(width, height, max_steps, &mut rng);
            run_watch(sim, &genome, &mut rng)?;
        }
    }

    Ok(())
}
