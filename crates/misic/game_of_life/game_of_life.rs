// 4 rules
// 1: Any living cell with less than 2 neighbors dies by under population
// 2: Any cell with 2 neighbors lives on (but nothing happens, then just survive)
// 3: Any living cell with exactly 3 neighbors creates a new cell randomly next to it
// 4: Any cell with more than 3 living cells next to it dies by over population

use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupKind {
    Random,
    Block,
    Bar,
    Glider,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    BoardSize(usize, usize),
    BoardSetup(SetupKind),
}

#[derive(Debug, Clone)]
pub struct Builder {
    data: Vec<Attribute>,
}

impl Builder {
    pub fn create() -> Self {
        Self { data: vec![] }
    }
    pub fn add(mut self, attrib: Attribute) -> Self {
        self.data.push(attrib);
        Self { data: self.data }
    }
    pub fn build(self) -> Option<Context> {
        let mut board_size = (0, 0);
        let mut data = None;
        let mut board_setup: Option<SetupKind> = None;

        for ca in self.data {
            match ca {
                Attribute::BoardSize(x_coord, y_coord) => {
                    // I know I can use NonZero... but idc right now
                    if x_coord == 0 || y_coord == 0 {
                        return None;
                    }
                    board_size = (x_coord, y_coord);
                    if data.is_none() {
                        data = Some(Vec::with_capacity(x_coord * y_coord));
                    }
                }
                Attribute::BoardSetup(kind) => board_setup = Some(kind),
            }
        }

        if let Some(mut data_) = data {
            for _ in 0..board_size.0 * board_size.1 {
                data_.push(Cell::Dead);
            }
            let mut ret = Context {
                board_size,
                data: data_,
                cnt: 0,
                last_gen_born: vec![],
                last_gen_killed: vec![],
            };

            if board_setup.is_none() {
                board_setup = Some(SetupKind::Bar);
            }
            match board_setup.unwrap() {
                SetupKind::Bar => {
                    ret.set_cell(3, 1, Cell::Alive);
                    ret.set_cell(3, 2, Cell::Alive);
                    ret.set_cell(3, 3, Cell::Alive);
                }
                SetupKind::Block => {
                    ret.set_cell(2, 1, Cell::Alive);
                    ret.set_cell(2, 2, Cell::Alive);
                    ret.set_cell(3, 1, Cell::Alive);
                    ret.set_cell(3, 2, Cell::Alive);
                }
                SetupKind::Random => {
                    for cell in &mut ret.data {
                        let is_alive = rand::random::<bool>();
                        if is_alive {
                            *cell = Cell::Alive;
                        }
                    }
                }
                SetupKind::Glider => {
                    ret.set_cell(2, 1, Cell::Alive);
                    ret.set_cell(3, 2, Cell::Alive);
                    ret.set_cell(1, 3, Cell::Alive);
                    ret.set_cell(2, 3, Cell::Alive);
                    ret.set_cell(3, 3, Cell::Alive);
                }
            }

            Some(ret)
        } else {
            None
        }
    }
}

// pretty sure that I've said this before, but it really is bad to
// have a whole byte for this: should definitely be packs of 8
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Alive = 1,
    #[default]
    Dead = 0,
}

impl Cell {
    pub fn is_alive(self) -> bool {
        match self {
            Self::Alive => true,
            Self::Dead => false,
        }
    }
    pub fn is_dead(self) -> bool {
        !self.is_alive()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellActionDirection {
    Up,
    Down,
    Left,
    Right,
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
}

impl CellActionDirection {
    pub fn from_u32(num: u8) -> Option<Self> {
        match num {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            2 => Some(Self::Up),
            3 => Some(Self::Down),
            4 => Some(Self::LeftUp),
            5 => Some(Self::LeftDown),
            6 => Some(Self::RightUp),
            7 => Some(Self::RightDown),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellAction {
    UnderPopulation,
    OverPopulation,
    ReproductionAlive,
    ReproductionDead,
    Pass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    // horizontal, vertical
    pub board_size: (usize, usize),
    data: Vec<Cell>,
    // how many generations/iterations
    pub cnt: u64,

    // these are the coords of the cells
    // which were borned last generation
    // and were killed last generation/tick
    // so if you need to update graphics
    // after doing a tick, you can use these
    last_gen_born: Vec<usize>,
    last_gen_killed: Vec<usize>,
}

fn symmetry_helper(val: usize, addend: usize, limit: usize) -> Option<usize> {
    let ret = val + addend;
    if ret < limit {
        Some(ret)
    } else {
        None
    }
}

impl Context {
    /// gets the slice of cells
    pub fn get_data(&self) -> &[Cell] {
        self.data.as_slice()
    }

    /// gets the indices of the cells killed last tick
    pub fn get_born(&self) -> &[usize] {
        self.last_gen_born.as_slice()
    }

    /// gets the indices of the cells killed last tick
    pub fn get_killed(&self) -> &[usize] {
        self.last_gen_killed.as_slice()
    }

    fn set_cell(&mut self, x_coord: usize, y_coord: usize, state: Cell) {
        let index = self.get_index(x_coord, y_coord);
        self.data[index] = state;
    }

    fn count_neighbors_dead(&self, x_coord: usize, y_coord: usize) -> CellAction {
        let mut cnt = 0;
        for cx in 0..8 {
            let direction = CellActionDirection::from_u32(cx).unwrap();
            let cell = self.get_relative(x_coord, y_coord, direction);
            if cell.is_alive() {
                // means that this should count towards our total
                cnt += 1;
            }
        }
        if cnt == 3 {
            // if a dead cell has exactly 3 living
            // neighbors, it will be made a living cell
            CellAction::ReproductionAlive
        } else {
            CellAction::ReproductionDead
        }
    }
    fn count_neighbors(&self, x_coord: usize, y_coord: usize) -> CellAction {
        // we have to look at each direction and check if
        // the cell there is alive or not
        if self.get(x_coord, y_coord).is_dead() {
            return self.count_neighbors_dead(x_coord, y_coord);
        }
        // if not dead, then we need to count to see
        // if the cell should move on or die
        let mut cnt = 0;
        for cx in 0..8 {
            let direction = CellActionDirection::from_u32(cx).unwrap();
            let cell = self.get_relative(x_coord, y_coord, direction);
            if cell.is_alive() {
                cnt += 1;
            }
        }

        // now that we have looked in each direction and
        // know how many cells are alive next to it, we
        // can do the actual rules of the game...
        match cnt {
            0..=1 => CellAction::UnderPopulation,
            2..=3 => CellAction::Pass,
            _ => CellAction::OverPopulation,
        }
    }

    /// gets the cell given index coords
    pub fn get_value(&self, x_coord: usize, y_coord: usize) -> Cell {
        let index = self.get_index(x_coord, y_coord);
        self.data[index]
    }
    /// gets the cell given index coords
    pub fn get(&self, x_coord: usize, y_coord: usize) -> &Cell {
        let index = self.get_index(x_coord, y_coord);
        &self.data[index]
    }
    /// gets the cell given index coords
    pub fn get_mut(&mut self, x_coord: usize, y_coord: usize) -> &mut Cell {
        let index = self.get_index(x_coord, y_coord);
        &mut self.data[index]
    }

    /// gets the relative cell given index coords and direction
    pub fn get_value_relative(
        &self,
        x_coord: usize,
        y_coord: usize,
        direction: CellActionDirection,
    ) -> Cell {
        let index = self.get_index_relative(x_coord, y_coord, direction);
        self.data[index]
    }
    /// gets the relative cell given index coords and direction
    pub fn get_relative(
        &self,
        x_coord: usize,
        y_coord: usize,
        direction: CellActionDirection,
    ) -> &Cell {
        let index = self.get_index_relative(x_coord, y_coord, direction);
        &self.data[index]
    }
    /// gets the relative cell given index coords and direction
    pub fn get_relative_mut(
        &mut self,
        x_coord: usize,
        y_coord: usize,
        direction: CellActionDirection,
    ) -> &mut Cell {
        let index = self.get_index_relative(x_coord, y_coord, direction);
        &mut self.data[index]
    }

    /// gets the index of the cell given index coords
    pub fn get_index(&self, x_coord: usize, y_coord: usize) -> usize {
        x_coord + y_coord * (self.board_size.0)
    }
    /// gets the relative index of the cell given index coords and direction
    pub fn get_index_relative(
        &self,
        x_coord: usize,
        y_coord: usize,
        direction: CellActionDirection,
    ) -> usize {
        match direction {
            CellActionDirection::Up => {
                let (x, y) = self.index_up(x_coord, y_coord);
                self.get_index(x, y)
            }
            CellActionDirection::Down => {
                let (x, y) = self.index_down(x_coord, y_coord);
                self.get_index(x, y)
            }
            CellActionDirection::Left => {
                let (x, y) = self.index_left(x_coord, y_coord);
                self.get_index(x, y)
            }
            CellActionDirection::Right => {
                let (x, y) = self.index_right(x_coord, y_coord);
                self.get_index(x, y)
            }
            CellActionDirection::LeftUp => {
                let (x, mut y) = self.index_left(x_coord, y_coord);
                y = self.index_up(x, y).1;
                self.get_index(x, y)
            }
            CellActionDirection::LeftDown => {
                let (x, mut y) = self.index_left(x_coord, y_coord);
                y = self.index_down(x, y).1;
                self.get_index(x, y)
            }
            CellActionDirection::RightUp => {
                let (x, mut y) = self.index_right(x_coord, y_coord);
                y = self.index_up(x, y).1;
                self.get_index(x, y)
            }
            CellActionDirection::RightDown => {
                let (x, mut y) = self.index_right(x_coord, y_coord);
                y = self.index_down(x, y).1;
                self.get_index(x, y)
            }
        }
    }

    fn index_up(&self, x_coord: usize, y_coord: usize) -> (usize, usize) {
        match y_coord.checked_sub(1) {
            Some(y) => (x_coord, y),
            None => (x_coord, self.board_size.1 - 1),
        }
    }
    fn index_down(&self, x_coord: usize, y_coord: usize) -> (usize, usize) {
        match symmetry_helper(y_coord, 1, self.board_size.1) {
            Some(y) => (x_coord, y),
            None => (x_coord, 0),
        }
    }
    fn index_left(&self, x_coord: usize, y_coord: usize) -> (usize, usize) {
        match x_coord.checked_sub(1) {
            Some(x) => (x, y_coord),
            None => (self.board_size.0 - 1, y_coord),
        }
    }
    fn index_right(&self, x_coord: usize, y_coord: usize) -> (usize, usize) {
        match symmetry_helper(x_coord, 1, self.board_size.0) {
            Some(x) => (x, y_coord),
            None => (0, y_coord),
        }
    }

    fn do_last_gen_buffer(&mut self) {
        for i in self.last_gen_killed.iter() {
            self.data[*i] = Cell::Dead;
        }
        for i in self.last_gen_born.iter() {
            self.data[*i] = Cell::Alive;
        }
    }

    /// does a generation of using Conway's Game of Life rules
    pub fn tick(&mut self) {
        self.last_gen_born.clear();
        self.last_gen_killed.clear();

        for cx in 0..self.board_size.0 {
            for cy in 0..self.board_size.1 {
                match self.count_neighbors(cx, cy) {
                    CellAction::UnderPopulation => {
                        // The Cell should die
                        self.last_gen_killed.push(self.get_index(cx, cy));
                    }
                    CellAction::OverPopulation => {
                        // again, the Cell should die
                        self.last_gen_killed.push(self.get_index(cx, cy));
                    }
                    CellAction::ReproductionAlive => {
                        // this is a dead cell and should be brought to life
                        self.last_gen_born.push(self.get_index(cx, cy));
                    }
                    CellAction::ReproductionDead => {
                        // this means that the dead cell didn't make
                        // the cut and should stay dead
                    }
                    CellAction::Pass => {
                        // Do nothing
                        continue;
                    }
                }
            }
        }

        self.do_last_gen_buffer();

        self.cnt += 1;
    }

    /// does an entire shift of all the alive cells
    /// using the provided direction
    pub fn shift(&mut self, direction: CellActionDirection) {
        self.last_gen_born.clear();
        self.last_gen_killed.clear();

        for cx in 0..self.board_size.0 {
            for cy in 0..self.board_size.1 {
                let cell = self.get(cx, cy);
                if cell.is_alive() {
                    let index = self.get_index_relative(cx, cy, direction);
                    self.last_gen_born.push(index);
                    self.last_gen_killed.push(self.get_index(cx, cy));
                }
            }
        }

        self.do_last_gen_buffer();
    }
}
