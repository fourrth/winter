use rand::Rng;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left")?,
            Direction::Right => write!(f, "right")?,
            Direction::Up => write!(f, "up")?,
            Direction::Down => write!(f, "down")?,
        }
        Ok(())
    }
}

impl TryFrom<u8> for Direction {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Direction::Left),
            1 => Ok(Direction::Right),
            2 => Ok(Direction::Up),
            3 => Ok(Direction::Down),
            val => Err(val),
        }
    }
}

/// This is the current state of the game
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Running,
    Dead,
}

pub enum BuildOptions {
    ArenaDim(u64, u64),
    StartingMoveDir(Direction),
    StartingHeadCoord(Coordinate),
}

pub struct Builder {
    setup: Vec<BuildOptions>,
}

impl Builder {
    pub fn create() -> Self {
        Self { setup: vec![] }
    }
    pub fn add(mut self, val: BuildOptions) -> Self {
        self.setup.push(val);
        self
    }

    pub fn build(self) -> Context {
        let mut arena_dim = (0, 0);
        let mut move_dir = Direction::Right;
        let mut start_coord = Coordinate((4, 4));
        for option in self.setup {
            match option {
                BuildOptions::ArenaDim(x, y) => {
                    arena_dim = (x, y);
                }
                BuildOptions::StartingMoveDir(dir) => {
                    move_dir = dir;
                }
                BuildOptions::StartingHeadCoord(c) => {
                    start_coord = c;
                }
            }
        }
        if arena_dim.0 == 0 && arena_dim.1 == 0 {
            arena_dim = (10, 10);
        }

        if start_coord.0 .0 >= arena_dim.0 {
            start_coord.0 .0 = arena_dim.0 / 2;
        }
        if start_coord.0 .1 >= arena_dim.1 {
            start_coord.0 .1 = arena_dim.1 / 2;
        }

        let tmp_arena = vec![Cell::Empty; (arena_dim.0 * arena_dim.1) as usize];
        let mut ret = Context {
            arena_dim,
            move_dir,
            snake_body: vec![],
            snake_head: start_coord,
            tmp_arena,
            food: Coordinate((0, 0)),
            add_part: false,
            score: 0,
        };
        ret.add_food();
        ret
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate(pub (u64, u64));
impl Coordinate {
    pub fn to_index_coord(self, arena_dim: (u64, u64)) -> usize {
        (self.0 .0 + self.0 .1 * arena_dim.0) as usize
    }
    pub fn shift_left(self, arena_dim: (u64, u64)) -> Self {
        if self.0 .0 == 0 {
            Self((arena_dim.0 - 1, self.0 .1))
        } else {
            Self((self.0 .0 - 1, self.0 .1))
        }
    }

    pub fn shift_right(self, arena_dim: (u64, u64)) -> Self {
        if self.0 .0 == arena_dim.0 - 1 {
            Self((0, self.0 .1))
        } else {
            Self((self.0 .0 + 1, self.0 .1))
        }
    }

    pub fn shift_up(self, arena_dim: (u64, u64)) -> Self {
        if self.0 .1 == 0 {
            Self((self.0 .0, arena_dim.1 - 1))
        } else {
            Self((self.0 .0, self.0 .1 - 1))
        }
    }

    pub fn shift_down(self, arena_dim: (u64, u64)) -> Self {
        if self.0 .1 == arena_dim.1 - 1 {
            Self((self.0 .0, 0))
        } else {
            Self((self.0 .0, self.0 .1 + 1))
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    SnakeHead,
    SnakeBody,
    Food,
}

/// Main snake context
#[derive(Debug)]
pub struct Context {
    pub arena_dim: (u64, u64),
    pub move_dir: Direction,
    pub snake_head: Coordinate,
    pub snake_body: Vec<Coordinate>,
    // should we add a new body part on next tick?
    pub add_part: bool,
    /// The coordinate where the food is on the board.
    /// This is guaranteed to not lie on top of the snake's body
    pub food: Coordinate,
    pub score: u64,
    tmp_arena: Vec<Cell>,
}

//0######
impl Context {
    // will add food to the board via the self.food variable
    fn add_food(&mut self) {
        let mut rng = rand::thread_rng();

        let mut rand_coord: Coordinate;
        // simplest way of doing it
        loop {
            rand_coord = Coordinate((
                rng.gen_range(0..self.arena_dim.0),
                rng.gen_range(0..self.arena_dim.1),
            ));
            if self.snake_head != rand_coord && self.snake_body.contains(&rand_coord) == false {
                break;
            } else {
                continue;
            }
        }
        self.food = rand_coord;
    }
    #[inline]
    pub fn tick(&mut self) -> GameState {
        let new_head = match self.move_dir {
            Direction::Left => self.snake_head.shift_left(self.arena_dim),
            Direction::Right => self.snake_head.shift_right(self.arena_dim),
            Direction::Up => self.snake_head.shift_up(self.arena_dim),
            Direction::Down => self.snake_head.shift_down(self.arena_dim),
        };
        // right here we define that the snake moves with it's head first
        // meaning we can't run through our own tail, even though
        // at the end of the tick the tail would be moved by one square
        if self.snake_body.contains(&new_head) {
            // then we just crashed into ourselves
            // the tmp arena thing is kept how it was,
            // The arena still should be fine as it isn't updated
            return GameState::Dead;
        }
        // if we got to here, we must have a valid new_head

        let push_val = if self.add_part {
            // means we want to add a body part
            // this means we should have eaten food
            self.score += 1;

            if let Some(&val) = self.snake_body.last() {
                Some(val)
            } else {
                // means that we have an empty body,
                // so we add our body part where the
                // head was
                Some(self.snake_head)
            }
        } else {
            None
        };

        for cx in (1..self.snake_body.len()).rev() {
            self.snake_body[cx] = self.snake_body[cx - 1];
        }
        if self.snake_body.is_empty() == false {
            self.snake_body[0] = self.snake_head;
        }

        self.snake_head = new_head;

        if let Some(val) = push_val {
            self.snake_body.push(val);
            self.add_part = false;
        }

        // now set the tmp arena thing
        debug_assert_eq!(
            self.tmp_arena.len() as u64,
            self.arena_dim.0 * self.arena_dim.1
        );
        self.tmp_arena.fill(Cell::Empty);
        self.tmp_arena[self.snake_head.to_index_coord(self.arena_dim)] = Cell::SnakeHead;
        for &ca in self.snake_body.iter() {
            self.tmp_arena[ca.to_index_coord(self.arena_dim)] = Cell::SnakeBody;
        }
        self.tmp_arena[self.food.to_index_coord(self.arena_dim)] = Cell::Food;

        // if we run into a food,
        // then on next tick
        // add part and change food location
        if self.snake_head == self.food {
            self.add_part = true;
            self.add_food();
        }

        GameState::Running
    }

    /// gets an iterator of the arena
    /// true if apart of the snake, false if not
    pub fn get_arena_iter(&self) -> impl Iterator<Item = &Cell> {
        //TODO: replace this with an iterator

        self.tmp_arena.iter()
    }
}
