use crate::ga::{GenomeOperator, RobotEvaluator, TournamentSelector};
use crate::genome::Genome;
use crate::grid::Grid;
use crate::simulation::Simulation;
use crate::tui::run_watch;
use anyhow::Result;
use clap::{Parser, Subcommand};
use gars::{Config, Engine};
use rand::distr::StandardUniform;
use rand::prelude::*;
use rand::rngs::ChaCha8Rng;
use std::path::PathBuf;

mod ga;
mod genome;
mod grid;
mod simulation;
mod tui;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the genetic algorithm to evolve the best robot
    Run {
        /// The width of the grid
        #[arg(long, default_value = "10")]
        width: usize,
        /// The height of the grid
        #[arg(long, default_value = "10")]
        height: usize,
        /// The probability a tile is a wall
        #[arg(long, default_value = "0.0")]
        wall_prob: f64,
        /// The probability a tile contains an object
        #[arg(long, default_value = "0.5")]
        obj_prob: f64,
        /// The robot population size
        #[arg(long, default_value = "200")]
        pop_size: usize,
        /// The number of generations to evolve
        #[arg(long, default_value = "1000")]
        generations: usize,
        /// The number of simulations to run to evaluate a robot
        #[arg(long, default_value = "100")]
        eval_runs: usize,
        /// The tournament size for selection
        #[arg(long, default_value = "5")]
        tournament: usize,
        /// The mutation rate
        #[arg(long, default_value = "0.02")]
        mutation: f64,
        /// The number of elite individual to keep for the next generation (must be less than the population size)
        #[arg(long, default_value = "10")]
        elite: usize,
        /// The initial energy of a robot
        #[arg(long, default_value = "200.0")]
        energy: f64,
        /// The reward to give for picking up an object
        #[arg(long, default_value = "10.0")]
        obj_reward: f64,
        /// The reward to give if a robot has any energy remaining
        #[arg(long, default_value = "1.0")]
        energy_reward: f64,
        /// The penalty to give for hitting a wall
        #[arg(long, default_value = "5.0")]
        wall_penalty: f64,
        /// The penalty for failing to pick up an object
        #[arg(long, default_value = "1.0")]
        pickup_penalty: f64,
        /// The penalty for idling
        #[arg(long, default_value = "0.0")]
        idle_penalty: f64,
        /// The file full path to save the best robot
        #[arg(long, default_value = "best.json")]
        output: PathBuf,
        /// The seed for the RNG for reproducible executions (optional)
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Watch how a previously evolved & saved robot cleans up a grid.
    /// Press 'Space' to execute a single step, 'a' to execute steps automatically and 'q' to quit.
    Watch {
        /// The file full path of a saved robot to watch
        #[arg(long)]
        input: PathBuf,
        /// The width of the grid
        #[arg(long, default_value = "10")]
        width: usize,
        /// The height of the grid
        #[arg(long, default_value = "10")]
        height: usize,
        /// The probability a tile is a wall
        #[arg(long, default_value = "0.0")]
        wall_prob: f64,
        /// The probability a tile contains an object
        #[arg(long, default_value = "0.5")]
        obj_prob: f64,
        /// The initial energy of a robot
        #[arg(long, default_value = "200.0")]
        energy: f64,
        /// The seed for the RNG for reproducible executions (optional)
        #[arg(long)]
        seed: Option<u64>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            width,
            height,
            wall_prob,
            obj_prob,
            pop_size,
            generations,
            eval_runs,
            tournament,
            mutation,
            elite,
            energy,
            obj_reward,
            energy_reward,
            wall_penalty,
            pickup_penalty,
            idle_penalty,
            output,
            seed,
        } => {
            let seed = seed.unwrap_or_else(|| rand::rng().sample(StandardUniform));
            let evaluator = RobotEvaluator {
                grid_with: width,
                grid_height: height,
                wall_prob,
                obj_prob,
                initial_energy: energy,
                eval_runs,
                obj_reward,
                energy_reward,
                wall_penalty,
                pickup_penalty,
                idle_penalty,
            };
            let operator = GenomeOperator {
                mutation_rate: mutation,
            };
            let selector = TournamentSelector { size: tournament };
            let config = Config {
                population_size: pop_size,
                max_generations: generations,
                elite_count: elite,
                seed,
            };

            let mut engine = Engine::new(config, evaluator, operator, selector);
            engine.run();

            println!("best fitness = {}", engine.best().unwrap().fitness);

            let json = serde_json::to_string_pretty(&engine.best().unwrap().genotype)?;
            std::fs::write(output, json)?;
        }
        Commands::Watch {
            input,
            width,
            height,
            wall_prob,
            obj_prob,
            energy,
            seed,
        } => {
            let reader = std::fs::File::open(input)?;
            let genome: Genome = serde_json::from_reader(reader)?;
            let seed = seed.unwrap_or_else(|| rand::rng().sample(StandardUniform));
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let grid = Grid::new_random(width, height, obj_prob, wall_prob, &mut rng);
            let sim = Simulation::new(grid, energy);
            run_watch(sim, &genome, &mut rng)?;
        }
    }
    Ok(())
}
