use crate::genome::{Action, Genome};
use crate::grid::{Direction, Grid, Tile};
use rand::distr::StandardUniform;
use rand::{Rng, RngExt};

pub struct Simulation {
    grid: Grid,
    x: usize,
    y: usize,
    initial_energy: f64,
    energy: f64,
    obj_collected: u32,
    step_count: usize,
    history: Vec<StepRecord>,
}

impl Simulation {
    #[must_use]
    pub fn new(grid: Grid, energy: f64) -> Self {
        Self {
            grid,
            x: 0,
            y: 0,
            initial_energy: energy,
            energy,
            obj_collected: 0,
            step_count: 0,
            history: vec![],
        }
    }

    /// Run the simulation until all objects are collected or the energy is depleted.
    pub fn run<R: Rng>(mut self, genotype: &Genome, rng: &mut R) -> SimulationHistory {
        while self.step(genotype, rng) {}
        SimulationHistory::new(
            self.initial_energy,
            self.energy,
            self.obj_collected,
            self.grid.remaining_obj,
            self.history,
        )
    }

    /// Perform an action.
    ///
    /// Return `true` if the simulation can continue, otherwise `false`.
    #[must_use]
    pub fn step<R: Rng>(&mut self, genotype: &Genome, rng: &mut R) -> bool {
        if self.energy <= 0.0 || self.grid.remaining_obj == 0 {
            false
        } else {
            self.step_count += 1;
            let action = Self::get_action(genotype, &self.grid, self.x, self.y);
            let cost = action.energy_cost();
            let step = match action {
                Action::Idle => StepResult::Idled,
                Action::Pickup => {
                    if self.grid.pickup(self.x, self.y) {
                        self.obj_collected += 1;
                        StepResult::PickedUp
                    } else {
                        StepResult::FailedPickup
                    }
                }
                Action::MoveNorth | Action::MoveSouth | Action::MoveEast | Action::MoveWest => {
                    let dir = Direction::try_from(action).unwrap();
                    self.update_pos(self.x, self.y, dir)
                }
                Action::MoveRandom => {
                    let dir = rng.sample(StandardUniform);
                    self.update_pos(self.x, self.y, dir)
                }
            };
            // subtract the action's energy cost
            self.energy -= cost;
            // keep track of the new state
            self.history.push(StepRecord {
                x: self.x,
                y: self.y,
                energy: self.energy,
                result: step,
            });
            true
        }
    }

    #[inline]
    fn update_pos(&mut self, x: usize, y: usize, dir: Direction) -> StepResult {
        if self.grid.peek(x, y, dir) == Tile::Wall {
            StepResult::HitWall
        } else {
            match dir {
                Direction::North => {
                    self.y -= 1;
                }
                Direction::South => {
                    self.y += 1;
                }
                Direction::East => {
                    self.x += 1;
                }
                Direction::West => {
                    self.x -= 1;
                }
                Direction::None => {}
            }
            StepResult::Moved(dir)
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn energy(&self) -> f64 {
        self.energy
    }

    pub fn obj_remaining(&self) -> u32 {
        self.grid.remaining_obj
    }

    pub fn obj_collected(&self) -> u32 {
        self.obj_collected
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    #[inline]
    fn get_action(genotype: &Genome, grid: &Grid, x: usize, y: usize) -> Action {
        let north = grid.peek(x, y, Direction::North);
        let south = grid.peek(x, y, Direction::South);
        let east = grid.peek(x, y, Direction::East);
        let west = grid.peek(x, y, Direction::West);
        let cur = grid.peek(x, y, Direction::None);

        // 5 digits base-3 value encoding
        let idx = north as usize
            + south as usize * 3
            + east as usize * 9
            + west as usize * 27
            + cur as usize * 81;
        genotype[idx]
    }
}

#[derive(Clone, Debug)]
pub struct SimulationHistory {
    pub initial_energy: f64,
    pub remaining_energy: f64,
    pub collected_obj: u32,
    pub remaining_obj: u32,
    pub steps: Vec<StepRecord>,
}

impl SimulationHistory {
    #[must_use]
    pub fn new(
        initial_energy: f64,
        remaining_energy: f64,
        collected_obj: u32,
        remaining_obj: u32,
        steps: Vec<StepRecord>,
    ) -> Self {
        Self {
            initial_energy,
            remaining_energy,
            collected_obj,
            remaining_obj,
            steps,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepRecord {
    pub x: usize,
    pub y: usize,
    pub energy: f64,
    pub result: StepResult,
}

#[derive(Copy, Clone, Debug)]
pub enum StepResult {
    /// No action taken, or failed to pick up an object (empty tile)
    Idled,
    /// Successfully picked up an object
    PickedUp,
    /// Failed to pick up an object (no object on position)
    FailedPickup,
    /// Failed to move (directional or random) due to a wall
    HitWall,
    /// Successfully moved in the given direction
    Moved(Direction),
}
