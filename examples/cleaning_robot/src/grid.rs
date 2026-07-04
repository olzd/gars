use rand::distr::{Distribution, StandardUniform};
use rand::{Rng, RngExt};
use crate::genome::Action;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Tile {
    #[default]
    Empty = 0,
    Wall = 1,
    Obj = 2,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
    #[default]
    None = 4,
}

impl TryFrom<Action> for Direction {
    type Error = &'static str;

    fn try_from(action: Action) -> Result<Self, Self::Error> {
        match action {
            Action::MoveNorth => Ok(Direction::North),
            Action::MoveSouth => Ok(Direction::South),
            Action::MoveEast => Ok(Direction::East),
            Action::MoveWest => Ok(Direction::West),
            Action::Idle | Action::Pickup => Ok(Direction::None),
            Action::MoveRandom => Err("cannot guess direction from random move")
        }
    }
}

impl Distribution<Direction> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        let n = rng.random_range(0..=4);
        match n {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            4 => Direction::None,
            _ => unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    pub remaining_obj: u32,
}

impl Grid {
    #[must_use]
    pub fn new_random<R: Rng>(width: usize, height: usize, obj_prob: f64, wall_prob: f64, rng: &mut R) -> Self {
        let mut tiles = vec![Tile::default(); width * height];
        let mut remaining_obj = 0;

        for y in 0..height {
            for x in 0..width {
                if x == 0 && y == 0 {
                    // starting point must be empty
                    continue;
                }
                let p: f64 = rng.sample(StandardUniform);
                if p < wall_prob {
                    tiles[y * width + x] = Tile::Wall;
                } else if p < wall_prob + obj_prob {
                    tiles[y * width + x] = Tile::Obj;
                    remaining_obj += 1;
                }
            }
        }

        Self {
            width,
            height,
            tiles,
            remaining_obj,
        }
    }

    /// Peek at a neighboring tile from the given (`x`, `y`) position and direction.<br>
    /// Return `Tile::Wall` if out-of-bounds.
    pub fn peek(&self, x: usize, y: usize, dir: Direction) -> Tile {
        match dir {
            Direction::North if y > 0 => self.tiles[(y - 1) * self.width + x],
            Direction::South if y + 1 < self.height => self.tiles[(y + 1) * self.width + x],
            Direction::East if x + 1 < self.width => self.tiles[y*self.width + x+1],
            Direction::West if x > 0 => self.tiles[y*self.width + x-1],
            Direction::None => self.tiles[y*self.width + x],
            _ => Tile::Wall,
        }
    }

    /// Try to pick up an obj at the given (`x`, `y`) position.<br>
    /// Return `true` if the pickup was successful, otherwise `false`.
    pub fn pickup(&mut self, x: usize, y: usize) -> bool {
        if self.tiles[y * self.width + x] == Tile::Obj {
            self.tiles[y * self.width + x] = Tile::Empty;
            self.remaining_obj -= 1;
            true
        } else {
            false
        }
    }
}
