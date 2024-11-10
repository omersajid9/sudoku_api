use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use rand::Rng;

#[derive(Debug, Clone)]
struct ModeType {
    width: usize,
    height: usize,
    lower_size: usize,
    higher_size: usize,
}

#[derive(Debug)]
struct SudokuError(String);

impl fmt::Display for SudokuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SudokuError {}

const DEFAULT_MODE: &str = "9";

lazy_static::lazy_static! {
    static ref MODES: HashMap<String, ModeType> = {
        let mut m = HashMap::new();
        m.insert("4".to_string(), ModeType { width: 2, height: 2, lower_size: 4, higher_size: 8 });
        m.insert("6".to_string(), ModeType { width: 3, height: 2, lower_size: 9, higher_size: 18 });
        m.insert("8".to_string(), ModeType { width: 2, height: 4, lower_size: 18, higher_size: 36 });
        m.insert("9".to_string(), ModeType { width: 3, height: 3, lower_size: 17, higher_size: 40 });
        m
    };
}

#[derive(Debug)]
pub struct Sudoku {
    pub grid: Vec<Vec<usize>>,
    pub mode: ModeType,
    pub block_size: usize,
    pub numbers: Vec<usize>,
}

impl Sudoku {
    pub fn new(grid: Option<Vec<Vec<usize>>>, mode: Option<String>) -> Self {
        let mode_key = match &grid {
            Some(g) => g.len().to_string(),
            None => mode.unwrap_or(DEFAULT_MODE.to_string()),
        };

        let mode = MODES.get(&mode_key).unwrap().clone();
        let block_size = mode.width * mode.height;
        let numbers: Vec<usize> = (1..=block_size).collect();
        let grid = grid.unwrap_or_else(|| Self::default_grid(block_size));

        Sudoku {
            grid,
            mode,
            block_size,
            numbers,
        }
    }

    fn default_grid(block_size: usize) -> Vec<Vec<usize>> {
        vec![vec![0; block_size]; block_size]
    }

    pub fn reset(&mut self) {
        self.grid = Self::default_grid(self.block_size);
    }

    pub fn set_board(&mut self, board: Vec<Vec<usize>>) {
        self.grid = board;
    }

    pub fn get_count(&self) -> usize {
        self.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&x| x > 0)
            .count()
    }

    pub fn get_difficulty(&self) -> &str {
        let count = self.get_count();
        match count {
            40..=81 => "Easy",
            25..=39 => "Medium",
            1..=24 => "Hard",
            _ => "Unknown",
        }
    }

    pub fn generate(&mut self) {
        self.reset();
        let mut rng = rand::thread_rng();

        let min_allowed_size = self.block_size / 3;
        let max_allowed_size = self.block_size - 2;
        let grid_cell_size = self.block_size.pow(2);

        let mut base_numbers = self.mode.lower_size;

        while base_numbers > 0 {
            let fill_x = rng.gen_range(0..self.block_size);
            let fill_y = rng.gen_range(0..self.block_size);
            let allowed_numbers = self.allowed_numbers(fill_x, fill_y);

            if allowed_numbers.len() > min_allowed_size {
                let random_index = rng.gen_range(0..allowed_numbers.len());
                if let Ok(_) = self.set(fill_x, fill_y, allowed_numbers[random_index]) {
                    base_numbers -= 1;
                }
            }
        }

        if !self.solve().is_some() {
            self.generate();
            return;
        }

        let mut dig_numbers = grid_cell_size - rng.gen_range(self.mode.lower_size..=self.mode.higher_size);

        while dig_numbers > 0 {
            let dig_x = rng.gen_range(0..self.block_size);
            let dig_y = rng.gen_range(0..self.block_size);

            if self.get(dig_x, dig_y) > 0 && self.allowed_numbers(dig_x, dig_y).len() < max_allowed_size {
                self.set(dig_x, dig_y, 0).unwrap();
                dig_numbers -= 1;
            }
        }
    }

    fn get(&self, x: usize, y: usize) -> usize {
        self.grid[y][x]
    }

    fn set(&mut self, x: usize, y: usize, value: usize) -> Result<usize, Box<dyn Error>> {
        if value > 0 {
            if self.get(x, y) == value {
                return Ok(value);
            }

            if !self.allowed_numbers_in_row(y).contains(&value) {
                return Err(Box::new(SudokuError(format!("{} is not allowed in the row {}", value, y))));
            }

            if !self.allowed_numbers_in_column(x).contains(&value) {
                return Err(Box::new(SudokuError(format!("{} is not allowed in the column {}", value, x))));
            }

            if !self.allowed_numbers_in_block(x, y).contains(&value) {
                return Err(Box::new(SudokuError(format!("{} is not allowed in the block", value))));
            }
        }

        self.grid[y][x] = value;
        Ok(value)
    }

    fn row(&self, y: usize) -> Vec<usize> {
        self.grid[y].clone()
    }

    fn column(&self, x: usize) -> Vec<usize> {
        self.grid.iter().map(|row| row[x]).collect()
    }

    fn allowed_numbers_in_row(&self, y: usize) -> Vec<usize> {
        let row = self.row(y);
        self.numbers.iter()
            .filter(|&&num| !row.contains(&num))
            .cloned()
            .collect()
    }

    fn allowed_numbers_in_column(&self, x: usize) -> Vec<usize> {
        let column = self.column(x);
        self.numbers.iter()
            .filter(|&&num| !column.contains(&num))
            .cloned()
            .collect()
    }

    fn allowed_numbers_in_block(&self, x: usize, y: usize) -> Vec<usize> {
        let bx = (x / self.mode.width) * self.mode.width;
        let by = (y / self.mode.height) * self.mode.height;
        
        let mut numbers_in_block = Vec::new();
        
        for i in 0..self.mode.width {
            for j in 0..self.mode.height {
                numbers_in_block.push(self.get(bx + i, by + j));
            }
        }

        self.numbers.iter()
            .filter(|&&num| !numbers_in_block.contains(&num))
            .cloned()
            .collect()
    }

    fn allowed_numbers(&self, x: usize, y: usize) -> Vec<usize> {
        let numbers_in_block = self.allowed_numbers_in_block(x, y);

        if numbers_in_block.len() > 1 {
            let numbers_in_row = self.allowed_numbers_in_row(y);
            let numbers_in_column = self.allowed_numbers_in_column(x);
            
            numbers_in_block.into_iter()
                .filter(|num| numbers_in_row.contains(num) && numbers_in_column.contains(num))
                .collect()
        } else {
            numbers_in_block
        }
    }

    fn empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, &num) in row.iter().enumerate() {
                if num == 0 {
                    cells.push((x, y));
                }
            }
        }
        cells
    }

    fn any_empty_cell(&self, allowed_numbers_length: Option<usize>) -> Option<(usize, usize)> {
        let mut min_length = allowed_numbers_length.unwrap_or(self.block_size + 1);
        let mut result = None;

        for (x, y) in self.empty_cells() {
            let length = self.allowed_numbers(x, y).len();
            if length < min_length {
                result = Some((x, y));
                min_length = length;
            }
            if length == 1 {
                break;
            }
        }

        result
    }

    fn is_solved(&self) -> bool {
        self.grid.iter().all(|row| row.iter().all(|&num| num > 0))
    }

    pub fn solve(&mut self) -> Option<Vec<Vec<usize>>> {
        if self.solve_ultimately() {
            Some(self.grid.clone())
        } else {
            None
        }
    }

    fn solve_ultimately(&mut self) -> bool {
        if self.is_solved() {
            return true;
        }

        if let Some((x, y)) = self.any_empty_cell(None) {
            let allowed_numbers = self.allowed_numbers(x, y);
            for &value in &allowed_numbers {
                if let Ok(_) = self.set(x, y, value) {
                    if self.solve_ultimately() {
                        return true;
                    }
                }
                let _ = self.set(x, y, 0);
            }
        }

        false
    }
}