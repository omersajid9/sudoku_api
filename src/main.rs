use std::sync::Arc;
use tokio::sync::Mutex;

mod sudoku; // Assuming the previous code is in sudoku.rs
use sudoku::Sudoku;

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    fn get_cell_ranges(&self) -> (usize, usize) {
        match self {
            Difficulty::Easy => (40, 81),    // More filled cells
            Difficulty::Medium => (25, 39),   // Medium amount of filled cells
            Difficulty::Hard => (17, 24),     // Fewer filled cells
        }
    }
}

#[derive(Clone)]
struct SudokuBoard {
    value: Vec<Vec<usize>>,
    difficulty: String,
    original_board: Arc<Mutex<Vec<Vec<usize>>>>,
}

impl SudokuBoard {
    async fn solution(&self) -> Option<Vec<Vec<usize>>> {
        let mut nsudoku = Sudoku::new(None, Some("9".to_string()));
        let board = self.original_board.lock().await.clone();
        nsudoku.set_board(board);
        nsudoku.solve()
    }
}

fn gen_board_with_difficulty(difficulty: Difficulty) -> SudokuBoard {
    let mut attempts = 0;
    let max_attempts = 100; // Prevent infinite loops
    
    loop {
        let mut sudoku = Sudoku::new(None, Some("9".to_string()));
        sudoku.generate();
        
        let count = sudoku.get_count();
        let (min_cells, max_cells) = difficulty.get_cell_ranges();
        
        if count >= min_cells && count <= max_cells {
            return SudokuBoard {
                value: sudoku.grid.clone(),
                difficulty: sudoku.get_difficulty().to_string(),
                original_board: Arc::new(Mutex::new(sudoku.grid)),
            };
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            // If we can't get exact difficulty after max attempts,
            // return the last generated board
            return SudokuBoard {
                value: sudoku.grid.clone(),
                difficulty: sudoku.get_difficulty().to_string(),
                original_board: Arc::new(Mutex::new(sudoku.grid)),
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Example of generating boards with different difficulties
    println!("Generating Easy Sudoku...");
    let easy_board = gen_board_with_difficulty(Difficulty::Easy);
    print_board_with_info(&easy_board.value, &easy_board.difficulty);
    
    println!("\nGenerating Medium Sudoku...");
    let medium_board = gen_board_with_difficulty(Difficulty::Medium);
    print_board_with_info(&medium_board.value, &medium_board.difficulty);
    
    println!("\nGenerating Hard Sudoku...");
    let hard_board = gen_board_with_difficulty(Difficulty::Hard);
    print_board_with_info(&hard_board.value, &hard_board.difficulty);
    
    // Example of solving one of the boards
    println!("\nSolving the Hard board...");
    if let Some(solution) = hard_board.solution().await {
        println!("\nSolution:");
        print_board(&solution);
    } else {
        println!("No solution found!");
    }
}

fn print_board_with_info(board: &Vec<Vec<usize>>, difficulty: &str) {
    println!("Difficulty: {}", difficulty);
    println!("Filled cells: {}", count_filled_cells(board));
    print_board(board);
}

fn count_filled_cells(board: &Vec<Vec<usize>>) -> usize {
    board.iter()
        .flat_map(|row| row.iter())
        .filter(|&&cell| cell != 0)
        .count()
}

fn print_board(board: &Vec<Vec<usize>>) {
    for (i, row) in board.iter().enumerate() {
        if i % 3 == 0 && i != 0 {
            println!("-------------------------");
        }
        for (j, &num) in row.iter().enumerate() {
            if j % 3 == 0 && j != 0 {
                print!("| ");
            }
            if num == 0 {
                print!(".  ");
            } else {
                print!("{:1}  ", num);
            }
        }
        println!();
    }
}