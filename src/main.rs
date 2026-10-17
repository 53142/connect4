mod minimax;

use colored::Colorize;
use std::{io::{self, Write}};

use crate::minimax::MiniMaxAI;

pub const BOARD_COLS: usize = 7;
const BOARD_ROWS: usize = 6;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Player {
    None = 0,
    One = 1,
    Two = 2,
}

// Board setup
// Stores board setup and logic for interacting with it
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Board {
    grid: [[Player; BOARD_COLS]; BOARD_ROWS],
    col_heights: [usize; BOARD_COLS],
}

impl Board {
    pub fn new() -> Self {
        Board {
            grid: [[Player::None; BOARD_COLS]; BOARD_ROWS],
            col_heights: [0; BOARD_COLS],
        }
    }
    
    // Ensure that column is valid and there is space to place a piece in that column
    pub fn is_valid_move(&self, col_index: usize) -> bool {
        col_index < BOARD_COLS && self.col_heights[col_index] < BOARD_ROWS
    }

pub fn move_piece(&mut self, col_index: usize, player: Player) -> Option<usize> {
        // Ensure valid movement
        if player != Player::None && self.is_valid_move(col_index) {
            let row_index = BOARD_ROWS - self.col_heights[col_index] - 1;
            // Move piece
            self.grid[row_index][col_index] = player;

            // Increment column height
            self.col_heights[col_index] += 1;
            return Some(row_index);
        }
        None
    }

    // Used to check for win at a specific position so we don't have to scan the whole board
    pub fn check_win_at(&self, row: usize, col: usize) -> Option<Player> {
        let player = self.grid[row][col];
        if player == Player::None {
            return None;
        }

        let directions = [
            (0, 1),   // horizontal
            (1, 0),   // vertical
            (1, 1),   // diagonal down-right
            (1, -1),  // diagonal down-left
        ];

        for (r_dir, c_dir) in directions {
            let mut count = 1;
            
            // Check positive direction
            let mut r = row as isize + r_dir;
            let mut c = col as isize + c_dir;
            while r >= 0 && r < BOARD_ROWS as isize && c >= 0 && c < BOARD_COLS as isize {
                if self.grid[r as usize][c as usize] == player {
                    count += 1;
                    r += r_dir;
                    c += c_dir;
                } else {
                    break;
                }
            }

            // Check negative direction
            let mut r = row as isize - r_dir;
            let mut c = col as isize - c_dir;
            while r >= 0 && r < BOARD_ROWS as isize && c >= 0 && c < BOARD_COLS as isize {
                if self.grid[r as usize][c as usize] == player {
                    count += 1;
                    r -= r_dir;
                    c -= c_dir;
                } else {
                    break;
                }
            }

            if count >= 4 {
                return Some(player);
            }
        }
        None
    }

    pub fn unmove_piece(&mut self, col_index: usize) -> Option<()> {
        if self.col_heights[col_index] > 0 {
            // Remove piece
            self.grid[BOARD_ROWS - self.col_heights[col_index]][col_index] = Player::None;

            // Decrement column height
            self.col_heights[col_index] -= 1;
            return Some(());
        }
        None
    }

    // Checks board for winner
    // Returns Some(Player) if game is complete with the winner (or Player::None if draw) and None if the game is not over yet
    pub fn check_for_winner(&self, move_number: &u8) -> Option<Player> {
        for i in 0..BOARD_ROWS {
            for j in 0..BOARD_COLS {
                let cell = self.grid[i][j];
                if cell == Player::None {
                    continue;
                }

                // Possible directions to make a Connect 4 in
                let directions = [
                    (0, 1), // horizontal
                    (1, 0), // vertical
                    (1, 1), // Upper Diagonal
                    (-1, 1), // Lower Diagonal
                ];

                for (row_offset, column_offset) in directions {
                    let mut num_in_row = 1;
                    let mut row_index  = i as isize + row_offset;
                    let mut column_index = j as isize + column_offset;
                    // Traverse to other cells in each direction to check if 4 in a row
                    while column_index < BOARD_COLS as isize && column_index >= 0 && row_index < BOARD_ROWS as isize && row_index >= 0 {
                        let new_cell = self.grid[row_index as usize][column_index as usize];
                        if new_cell == cell {
                            num_in_row += 1;
                        } else {
                            break;
                        }
                        if num_in_row == 4 {
                            return Some(cell)
                        }
                        row_index += row_offset;
                        column_index += column_offset;
                    }
                }
            }
        }

        // Draw if no winner and board full
        if *move_number == BOARD_ROWS as u8 * BOARD_COLS as u8 {
            return Some(Player::None);
        }

        None
    }

    // Heuristic evaluation function for minimax
    // Positive is better for player 1
    // Negative is better for player 2
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;
        
        // Center preference
       let center_col = BOARD_COLS / 2;
        for row in 0..BOARD_ROWS {
            match self.grid[row][center_col] {
                Player::One => score += 6,
                Player::Two => score -= 6,
                Player::None => {}
            }
        }

        // Evaluate the 69 scanning windows
        // Horizontal
        for row in 0..BOARD_ROWS {
            for col in 0..(BOARD_COLS - 3) {
                score += self.evaluate_window([self.grid[row][col], self.grid[row][col+1], self.grid[row][col+2], self.grid[row][col+3]]);
            }
        }

        // Vertical
        for row in 0..(BOARD_ROWS - 3) {
            for col in 0..BOARD_COLS {
                score += self.evaluate_window([self.grid[row][col], self.grid[row+1][col], self.grid[row+2][col], self.grid[row+3][col]]);
            }
        }

        // Lower Diagonal
        for row in 3..BOARD_ROWS {
            for col in 0..BOARD_COLS - 3 {
                score += self.evaluate_window([self.grid[row][col], self.grid[row-1][col+1], self.grid[row-2][col+2], self.grid[row-3][col+3]]);
            }
        }

        // Upper Diagonal
        for row in 0..BOARD_ROWS - 3 {
            for col in 0..BOARD_COLS - 3 {
                score += self.evaluate_window([self.grid[row][col], self.grid[row+1][col+1], self.grid[row+2][col+2], self.grid[row+3][col+3]]);
            }
        }

        score
    }

    // Simple function to evaluate set of 4 cells to determine a score
    fn evaluate_window(&self, window: [Player; 4]) -> i32 {
        let empty_cells = window.iter().filter(|&&p| p == Player::None).count();
        let player_1_cells = window.iter().filter(|&&p| p == Player::One).count();
        let player_2_cells = window.iter().filter(|&&p| p == Player::Two).count();
        
        let mut score = 0;

        // Favor combinations close to 4 in row
        if player_1_cells == 3 && empty_cells == 1 {
            score += 1_000;
        }
        if player_2_cells == 3 && empty_cells == 1 {
            score -= 1_000;
        }
        if player_1_cells == 2 && empty_cells == 2 {
            score += 10;
        }
        if player_2_cells == 2  && empty_cells == 2 {
            score -= 10;
        }

        if player_1_cells == 1 && empty_cells == 3 {
            score += 2;
        }
        if player_2_cells == 1  && empty_cells == 3 {
            score -= 2;
        }

        score
    }

    // Return all valid columns to place a piece in
    pub fn get_valid_moves(&self) -> Vec<usize> {
        (0..BOARD_COLS).filter(|&c| self.is_valid_move(c)).collect()
    }

    // Return a fancy image of the board for rendering in the terminal
    pub fn print_board(&self) -> String {
        let mut board_string = String::new();
        for board_row in self.grid { 
            // Display each cell in row
            let mut display_row = String::new();
            
            for cell in board_row.iter() {
                display_row.push_str(&"|".blue().to_string());
                if cell == &Player::None {
                    display_row.push_str(&"o".white().to_string());
                } else if cell == &Player::One {
                    display_row.push_str(&"●".red().to_string());
                } else {
                    display_row.push_str(&"●".yellow().to_string());
                }
            }
            // Add final pipe
            display_row.push_str(&"|".blue().to_string());
            board_string.push_str(&format!("{}{}\n", " ".repeat((40-15)/2), display_row));
        }
        
        // Add letters at bottom
        board_string.push_str(&format!(
            "{}{} {} {} {} {} {} {}\n",
            " ".repeat((40-13)/2),
            "A".yellow().bold(),
            "B".yellow().bold(),
            "C".yellow().bold(),
            "D".yellow().bold(),
            "E".yellow().bold(),
            "F".yellow().bold(),
            "G".yellow().bold()
        ));
        board_string
    }
}

enum PlayerInput {
    Column(char),
    Quit,
    Invalid,
}

fn try_parse_player_input(player_input: String) -> PlayerInput {
    let player_input = player_input.trim().to_lowercase();
    let mut chars = player_input.chars();
    let Some(first_char) = chars.next() else {
        return PlayerInput::Invalid
    };
    // Valid responses only have 1 char
    // If chars has multiple characters, return None
    if chars.next().is_some() {
        return PlayerInput::Invalid
    }
    // Ensure user input is alphabetical character in correct range
    if first_char >= 'a' && first_char < (b'a' + BOARD_COLS as u8) as char {
        PlayerInput::Column(first_char)
    } else if first_char == 'q' {
        PlayerInput::Quit
    } else {
        PlayerInput::Invalid
    }
}

fn main() {
    let mut move_number: u8 = 0;

    // Player 1 starts first
    let mut player_1_turn = true;

    // Create board with elements in each row set to 0
    let mut board = Board::new();

    let mut playing_computer = false;
    let mut computer_turn_1 = false;

    let mut minimax = MiniMaxAI::new();

    // Clear terminal
    print!("\x1B[2J\x1B[1;1H");

    // Fancy Title
    println!("{:^40}", "Connect 4".magenta().bold().underline());

    println!("Type a number\n[1] Play Against Computer\n[2] Play Against Person");
    loop {
        let mut player_input = String::new();
        io::stdin()
            .read_line(&mut player_input)
            .expect("Failed to read user input");
        if let Ok(num) = player_input.trim().parse::<i32>() {
            if num == 1 {
                playing_computer = true;
                break;
            } else if num == 2 {
                break;
            }
        }
        continue;
    }

    // Clear terminal
    print!("\x1B[2J\x1B[1;1H");

    if playing_computer {
        println!("Which player do you want to be?\n[1] Player One (You start)\n[2] Player 2 (The computer starts)");
        loop {
            let mut player_input = String::new();
            io::stdin()
                .read_line(&mut player_input)
                .expect("Failed to read user input");
            if let Ok(num) = player_input.trim().parse::<i32>() {
                if num == 1 {
                    computer_turn_1 = false;
                    break;
                } else if num == 2 {
                    computer_turn_1 = true;
                    break;
                }
            }
            continue;
        }
    }


    // Start gameplay loop
    loop {
        // Clear terminal
        print!("\x1B[2J\x1B[1;1H");

        // Fancy Title
        println!("{:^40}", "Connect 4".magenta().bold().underline());

        // Show board
        println!("{}", board.print_board());
        println!("Controls: {} Drop Piece | {} Quit\n", "[A-G]".blue(), "[Q]".blue());
        println!("Current Turn: {}", if player_1_turn { "Player 1".to_string().red().bold() } else { "Player 2".to_string().yellow().bold() });
        // User input loop
        loop {
            if playing_computer && computer_turn_1 == player_1_turn { // If playing computer & computer turn
                let player = if player_1_turn { &Player::One} else { &Player::Two };
                let (_, col_index) = minimax.minimax(&mut board, 15, &player);
                board.move_piece(col_index.unwrap(), *player);
                break;
            } else {
                print!("Enter your move: ");
                // Flush buffer to show above print statement
                io::stdout().flush().expect("Failed to flush stdout");
                let mut player_input = String::new();
                io::stdin()
                    .read_line(&mut player_input)
                    .expect("Failed to read user input");
                match try_parse_player_input(player_input) {
                    PlayerInput::Column(c) => {
                        let column_index = (c as u8) - ('a' as u8);
                        if let None = board.move_piece(column_index as usize, if player_1_turn { Player::One} else { Player::Two }) {
                            continue; // Ask for input again
                        }
                        break;
                    }
                    PlayerInput::Quit => {
                        println!("{}", "Exiting game".yellow());
                        return;
                    }
                    PlayerInput::Invalid => {
                        println!("{} Enter a letter from {} to {} or {} to Quit", "Invalid Input!".red().bold(), "A".yellow().bold(), "G".yellow().bold(), "Q".yellow().bold());
                        continue;
                    } 
                }
            }
        }
        move_number += 1;
        // Check for winner
        if let Some(winner) = board.check_for_winner(&move_number) {
            // Clear terminal
            print!("\x1B[2J\x1B[1;1H");
            // Draw final board
            println!("{}", board.print_board());

            println!("{}", "GAME OVER".green().bold());
            if winner == Player::One {
                println!("{} won! 🎉", "Player 1".red().bold());
            } else if winner == Player::Two {
                println!("{} won! 🎉", "Player 2".yellow().bold());
            } else {
                println!("It was a draw!");
            }
            break;
        }

        player_1_turn = !player_1_turn;
    }
}
