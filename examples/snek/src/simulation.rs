use crate::genome::{Action, Genome};
use rand::{Rng, RngExt};
use std::collections::VecDeque;

pub struct Simulation {
    pub width: usize,
    pub height: usize,
    pub snake: VecDeque<(usize, usize)>,
    pub food: (usize, usize),
    pub direction: Direction,
    pub steps: usize,
    pub max_steps: usize,
}

impl Simulation {
    pub fn new(width: usize, height: usize, max_steps: usize, rng: &mut impl Rng) -> Self {
        // place the snake of length 3 in the middle, moving right
        let head_x = width / 2;
        let head_y = height / 2;
        let mut snake = VecDeque::<(usize, usize)>::new();
        snake.push_front((head_x - 2, head_y));
        snake.push_front((head_x - 1, head_y));
        snake.push_front((head_x, head_y));
        // randomly place a single food element on the grid, avoiding the snake
        let food = Self::random_food(&snake, width, height, rng);
        Self {
            width,
            height,
            snake,
            food,
            direction: Direction::Right,
            steps: 0,
            max_steps,
        }
    }

    /// Run the simulation until the snake dies or [`max_steps`][`Self::max_steps`] are reached.
    pub fn run(&mut self, genotype: &Genome, rng: &mut impl Rng) {
        while self.step(genotype, rng) {}
    }

    /// Apply a relative action: move the snake and handle food/collision.
    ///
    /// Returns `true` if the simulation continues, `false` if dead or max steps reached.
    #[must_use]
    pub fn step(&mut self, genotype: &Genome, rng: &mut impl Rng) -> bool {
        let perception = self.perception();
        let action = genotype.forward(&perception);
        let new_dir = match action {
            Action::Straight => self.direction,
            Action::TurnLeft => self.direction.turn_left(),
            Action::TurnRight => self.direction.turn_right(),
        };
        self.direction = new_dir;

        let (dx, dy) = new_dir.delta();
        let head = self.snake[0];
        let x = head.0.checked_add_signed(dx).unwrap_or(self.width);
        let y = head.1.checked_add_signed(dy).unwrap_or(self.height);
        // new head
        let new_head = (x, y);

        // check borders collision
        if x >= self.width || y >= self.height {
            return false;
        }

        // check self collision (ignore tail because it will move, unless food is eaten)
        if self.snake.contains(&new_head) {
            // safe if the new head won't reach the tail or if it is but the tail will move
            if !self.snake.is_empty() && new_head != *self.snake.back().unwrap() {
                return false;
            }
        }

        // move: add new head
        self.snake.push_front(new_head);

        // check food
        if new_head == self.food {
            // the snake grows: don't remove the tail
            self.food = Self::random_food(&self.snake, self.width, self.height, rng);
        } else {
            self.snake.pop_back();
        }

        self.steps += 1;
        if self.steps > self.max_steps {
            return false;
        }
        true
    }

    /// Build the input vector for the snake's current state.
    fn perception(&self) -> Vec<f64> {
        // check for dangerous position relative to current heading
        let straight_dir = self.direction;
        let left_dir = straight_dir.turn_left();
        let right_dir = straight_dir.turn_right();
        let danger_straight =
            Self::is_position_dangerous(self.width, self.height, &self.snake, straight_dir);
        let danger_left =
            Self::is_position_dangerous(self.width, self.height, &self.snake, left_dir);
        let danger_right =
            Self::is_position_dangerous(self.width, self.height, &self.snake, right_dir);

        // food direction relative to head position
        let head = self.snake[0];
        let food = self.food;
        let food_left = food.0 < head.0;
        let food_right = food.0 > head.0;
        let food_up = food.1 < head.1;
        let food_down = food.1 > head.1;

        // current head direction
        let (up, down, left, right) = match straight_dir {
            Direction::Up => (1.0, 0.0, 0.0, 0.0),
            Direction::Down => (0.0, 1.0, 0.0, 0.0),
            Direction::Left => (0.0, 0.0, 1.0, 0.0),
            Direction::Right => (0.0, 0.0, 0.0, 1.0),
        };

        // tail direction relative to the head
        let tail = self.snake.back().unwrap();
        let tail_left = tail.0 < head.0;
        let tail_right = tail.0 > head.0;
        let tail_up = tail.1 < head.1;
        let tail_down = tail.1 > head.1;

        // compute normalized Manhattan distance from head to food
        let (hx, hy) = head;
        let (fx, fy) = food;
        let food_dist =
            (hx.abs_diff(fx) + hy.abs_diff(fy)) as f64 / self.width.max(self.height) as f64;

        vec![
            food_dist,
            danger_straight.into(),
            danger_left.into(),
            danger_right.into(),
            food_left.into(),
            food_right.into(),
            food_up.into(),
            food_down.into(),
            tail_left.into(),
            tail_right.into(),
            tail_up.into(),
            tail_down.into(),
            up,
            down,
            left,
            right,
        ]
    }

    #[inline]
    fn random_food(
        snake: &VecDeque<(usize, usize)>,
        width: usize,
        height: usize,
        rng: &mut impl Rng,
    ) -> (usize, usize) {
        loop {
            let x = rng.random_range(0..width);
            let y = rng.random_range(0..height);
            if !snake.contains(&(x, y)) {
                return (x, y);
            }
        }
    }

    #[inline]
    fn is_position_dangerous(
        width: usize,
        height: usize,
        snake: &VecDeque<(usize, usize)>,
        dir: Direction,
    ) -> bool {
        let head = snake[0];
        let (dx, dy) = dir.delta();
        let new_x = head.0.checked_add_signed(dx).unwrap_or(width);
        let new_y = head.1.checked_add_signed(dy).unwrap_or(height);
        if new_x >= width || new_y >= height {
            return true;
        }
        snake.contains(&(new_x, new_y))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    #[must_use]
    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    #[must_use]
    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    #[must_use]
    pub fn delta(self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}
