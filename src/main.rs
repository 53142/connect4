mod minimax;

use colored::Colorize;
use std::{io::{self, Write}};

use crate::minimax::MiniMaxAI;

pub const BOARD_COLS: usize = 7;
const BOARD_ROWS: usize = 6;

// Game difficulty, encoded with time for computer to think for each difficulty
#[derive(Copy, Clone)]
#[repr(u128)]
enum GameDifficulty {
    Easy = 5,
    Medium = 85,
    Hard = 250,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Player {
    None = 0,
    One = 1,
    Two = 2,
}

// Bitboard setup
// 7 bits per Column
// Bit 6 - unused bc base 2 stuff
// Bit 5 - 6th cell from bottom
// Bit 4 - 5th cell from bottom
// ...
// Bit 0 - Bottom-most cell

// How much to shift to get to new column
const STRIDE: usize = BOARD_ROWS + 1;

const WIN_SHIFTS: [u32; 4] = [
    1, // Shifting 1 bit moves vertically
    7, // Shifting 7 bits moves horizontally
    6, // Shifting 6 bits moves diagonal
    8 // Shifting 8 bits moves diagonal in opposite dir.
];

// Constant for number of windows based on board dimensions
const WINDOW_COUNT: usize =
    BOARD_ROWS * (BOARD_COLS - 3)
    + (BOARD_ROWS - 3) * BOARD_COLS
    + 2 * (BOARD_ROWS - 3) * (BOARD_COLS - 3);

// Get bit for cell based on row and col
const fn cell_bit(row: usize, col: usize) -> u64 {
    1u64 << (col * STRIDE + (BOARD_ROWS - 1 - row))
}

// Function to return the window mask for a desired window
const fn window_mask(
    row: usize,
    col: usize,
    delta_row: isize,
    delta_col: isize,
) -> u64 {
    let mut mask = 0u64;
    let mut i = 0;

    while i < 4 {
        let r = (row as isize + i as isize * delta_row) as usize;
        let c = (col as isize + i as isize * delta_col) as usize;

        // Add to mask
        mask |= cell_bit(r, c);
        i += 1;
    }

    mask
}

// Generate winning window masks
const fn build_window_masks() -> [u64; WINDOW_COUNT] {
    let mut masks = [0u64; WINDOW_COUNT];
    let mut n = 0;

    // Masks for horizontal windows
    let mut row = 0;
    while row < BOARD_ROWS {
        let mut col = 0;
        while col + 3 < BOARD_COLS {
            masks[n] = window_mask(row, col, 0, 1);
            n += 1;
            col += 1;
        }
        row += 1;
    }

    // Masks for vertical windows
    row = 0;
    while row + 3 < BOARD_ROWS {
        let mut col = 0;
        while col < BOARD_COLS {
            masks[n] = window_mask(row, col, 1, 0);
            n += 1;
            col += 1;
        }
        row += 1;
    }

    // Masks for lower diagonal windows
    row = 0;
    while row + 3 < BOARD_ROWS {
        let mut col = 0;
        while col + 3 < BOARD_COLS {
            masks[n] = window_mask(row, col, 1, 1);
            n += 1;
            col += 1;
        }
        row += 1;
    }

    // Masks for upper diagonal windows
    row = 3;
    while row < BOARD_ROWS {
        let mut col = 0;
        while col + 3 < BOARD_COLS {
            masks[n] = window_mask(row, col, -1, 1);
            n += 1;
            col += 1;
        }
        row += 1;
    }

    masks
}

// Generate the window masks at compile time
static WINDOW_MASKS: [u64; WINDOW_COUNT] = build_window_masks();

// Six cells in center column
// Add offset to center of board 
const CENTER_COLUMN_MASK: u64 = ((1u64 << BOARD_ROWS) - 1) << ((BOARD_COLS / 2) * STRIDE);

// Board setup
// Stores board setup and logic for interacting with it
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Board {
    player_one: u64, // Bitboard for storing player one
    player_two: u64, // Bitboard for storing player 2
    col_heights: [usize; BOARD_COLS], // heights of columns for easier calculation
}

impl Board {
    pub fn new() -> Self {
        Board {
            player_one: 0,
            player_two: 0,
            col_heights: [0; BOARD_COLS],
        }
    }
    
    // Ensure that column is valid and there is space to place a piece in that column
    pub fn is_valid_move(&self, col_index: usize) -> bool {
        col_index < BOARD_COLS && self.col_heights[col_index] < BOARD_ROWS
    }

    pub fn move_piece(&mut self, col_index: usize, player: Player) -> Option<usize> {
        // Ensure valid movement
        if player == Player::None || !self.is_valid_move(col_index) {
            return None;
        }

        // Bits to shift for height adjustment
        let height = self.col_heights[col_index] as usize;
        // Base index for column
        let base_bit_index = col_index * STRIDE;

        // Shift to proper bit position
        let bit: u64 = 1 << (base_bit_index + height);

        // Set bit with 'or'
        match player {
            Player::One => self.player_one |= bit,
            Player::Two => self.player_two |= bit,
            Player::None => return None,
        }

        self.col_heights[col_index] += 1;

        return Some(BOARD_ROWS - 1 - height)
    }

    pub fn unmove_piece(&mut self, col_index: usize) -> Option<()> {
        if col_index >= BOARD_COLS || self.col_heights[col_index] == 0 {
            return None;
        }

        let height = self.col_heights[col_index] - 1;
        let bit_index = col_index * STRIDE + height;
        let bit = 1u64 << bit_index;

        if self.player_one & bit != 0 {
            self.player_one &= !bit;
        } else if self.player_two & bit != 0 {
            self.player_two &= !bit;
        } else {
            return None;
        }

        self.col_heights[col_index] -= 1;
        Some(())
    }

    // Convert row and col to bit and return the state of that cell.
    fn cell_at(&self, row: usize, col: usize) -> Player {
        let bit_index = col * STRIDE + (BOARD_ROWS - 1 - row);
        let bit = 1u64 << bit_index;

        if self.player_one & bit != 0 {
            Player::One
        } else if self.player_two & bit != 0 {
            Player::Two
        } else {
            Player::None
        }
    }

    // Checks board for winner
    // Returns Some(Player) if game is complete with the winner (or Player::None if draw) and None if the game is not over yet
    pub fn check_for_winner(&self) -> Option<Player> {
        if self.has_four(self.player_one) {
            return Some(Player::One);
        }

        if self.has_four(self.player_two) {
            return Some(Player::Two);
        }

        // If all columns full, than board is full and game is a draw
        if self.col_heights.iter().all(|&height| height == BOARD_ROWS) {
            return Some(Player::None);
        }

        None
    }

    // Heuristic evaluation function for minimax
    // Positive is better for player 1
    // Negative is better for player 2
    pub fn evaluate(&self) -> i32 {
        // Center-column preference based on number of pieces in center for each player
        let player1_center_pieces = (self.player_one & CENTER_COLUMN_MASK).count_ones() as i32;
        let player2_center_pieces = (self.player_two & CENTER_COLUMN_MASK).count_ones() as i32;

        let mut score = (player1_center_pieces - player2_center_pieces) * 6;

        // Evaluate all 69 four-cell windows using bitmasks
        for &mask in WINDOW_MASKS.iter() {
            // Remove bits outside window and count all pieces for player 1
            let player1_pieces = (self.player_one & mask).count_ones();

            // Remove bits outside window and count all pieces for player 2
            let player2_pieces = (self.player_two & mask).count_ones();

            // Window with both player's pieces can't be made into a Connect 4 so it doesn't matter
            if player1_pieces > 0 && player2_pieces > 0 {
                continue;
            }

            score += match (player1_pieces, player2_pieces) {
                (3, 0) => 1_000,
                (0, 3) => -1_000,
                (2, 0) => 10,
                (0, 2) => -10,
                (1, 0) => 2,
                (0, 1) => -2,
                _ => 0,
            };
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
        for row in 0..BOARD_ROWS { 
            // Display each cell in row
            let mut display_row = String::new();
            
            for col in 0..BOARD_COLS {
                display_row.push_str(&"|".blue().to_string());
                match self.cell_at(row, col) {
                    Player::None => display_row.push_str(&"o".white().to_string()),
                    Player::One => display_row.push_str(&"●".red().to_string()),
                    Player::Two => display_row.push_str(&"●".yellow().to_string()),
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

    // Detect win given player pieces bits quickly using bitshifts
    fn has_four(&self, bits: u64) -> bool {
        // Check all winning bit shift
        for shift in WIN_SHIFTS {
            // Find same color pieces that are next to each other
            let pairs = bits & (bits >> shift);

            // Check if there is another adjacent pair in the same direction aka 4 same color pieces in a row
            if pairs & (pairs >> (2 * shift)) != 0 {
                return true;
            }
        }
        false
    }

    // Check if player won with their bitmasks
    pub fn has_won(&self, player: Player) -> bool {
        match player {
            Player::One => self.has_four(self.player_one),
            Player::Two => self.has_four(self.player_two),
            Player::None => false,
        }
    }
}

// Player input types
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
    // Valid inputs only have 1 char
    // If there are multiple characters in the input, return None bc invalid
    if chars.next().is_some() {
        return PlayerInput::Invalid
    }
    // Ensure user input is a letter in the range of possible moves
    if first_char >= 'a' && first_char < (b'a' + BOARD_COLS as u8) as char {
        PlayerInput::Column(first_char)
    } else if first_char == 'q' {
        PlayerInput::Quit
    } else {
        PlayerInput::Invalid
    }
}

fn main() {
    // Player 1 starts first
    let mut player_1_turn = true;

    // Create board with elements in each row set to 0
    let mut board = Board::new();

    let mut playing_computer = false;
    let mut computer_turn_1 = false;
    let mut game_difficulty = GameDifficulty::Easy;

    let mut minimax = MiniMaxAI::new();

    // Clear terminal
    print!("\x1B[2J\x1B[1;1H");

    // Fancy Title
    println!("{:^40}", "Connect 4".magenta().bold().underline());

    println!("{}\n{} Play Against Computer\n{} Play Against Person", "Who do you want to play?".underline().bold(), "[1]".red().bold(), "[2]".yellow().bold());
    loop {
        let mut player_input = String::new();
        io::stdin()
            .read_line(&mut player_input)
            .expect("Failed to read user input");
        player_input = player_input.trim().to_lowercase().to_string();
        if player_input == "q" {
            println!("{}", "Exiting game".yellow());
            return;
        }
        if let Ok(num) = player_input.parse::<i32>() {
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
        // Fancy Title
        println!("{:^40}", "Connect 4".magenta().bold().underline());

        // Mode selection prompt
        println!("{}\n{} Player 1 (You start)\n{} Player 2 (Computer starts)", "Which player do you want to be?".underline().bold(), "[1]".red().bold(), "[2]".yellow().bold());
        loop {
            let mut player_input = String::new();
            io::stdin()
                .read_line(&mut player_input)
                .expect("Failed to read user input");
            player_input = player_input.trim().to_lowercase().to_string();
            if player_input == "q" {
                println!("{}", "Exiting game".yellow());
                return;
            }
            if let Ok(num) = player_input.parse::<i32>() {
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

        // Clear terminal
        print!("\x1B[2J\x1B[1;1H");

        // Fancy Title
        println!("{:^40}", "Connect 4".magenta().bold().underline());

        // Difficulty prompt
        println!("{}\n{} Easy\n{} Medium\n{} Hard", "Which difficulty do you want?".underline().bold(), "[1]".green().bold(), "[2]".yellow().bold(), "[3]".red().bold());
        loop {
            let mut player_input = String::new();
            io::stdin()
                .read_line(&mut player_input)
                .expect("Failed to read user input");
            player_input = player_input.trim().to_lowercase().to_string();
            if player_input == "q" {
                println!("{}", "Exiting game".yellow());
                return;
            }
            if let Ok(num) = player_input.parse::<i32>() {
                if num == 1 {
                    game_difficulty = GameDifficulty::Easy;
                } else if num == 2 {
                    game_difficulty = GameDifficulty::Medium;
                } else if num == 3 {
                    game_difficulty = GameDifficulty::Hard;
                } else {
                    continue;
                }
                break;
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
                let (_, col_index) = minimax.minimax(&mut board, game_difficulty as u128, &player);
                board.move_piece(col_index.expect("Unable to find best move?"), *player);
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

        // Check for winner
        if let Some(winner) = board.check_for_winner() {
            // Clear terminal
            print!("\x1B[2J\x1B[1;1H");
            // Draw final board
            println!("{}", board.print_board());

            println!("{}", "GAME OVER".green().bold().underline());
            if playing_computer {
                // If winner is computer give message that user lost
                if winner == if computer_turn_1 { Player::One } else { Player::Two } {
                    println!("{}", "You lost!".red().bold());
                } else if winner == Player::None {
                    println!("{}", "You drew against the computer! 🎉".yellow().bold());
                } else {
                    println!("{}", "You beat the computer! 🎉".red().bold());
                }                
            } else {
                if winner == Player::One {
                    println!("{} won! 🎉", "Player 1".red().bold());
                } else if winner == Player::Two {
                    println!("{} won! 🎉", "Player 2".yellow().bold());
                } else {
                    println!("It was a draw!");
                }
            }
            break;
        }
        player_1_turn = !player_1_turn;
    }
}
