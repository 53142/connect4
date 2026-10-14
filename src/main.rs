use colored::Colorize;
use std::{error::Error, io::{self, Write}};

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

// Board setup
// 0 represents no player, 1 represents player 1, and 2 represents player 2
type Board = [[u8; BOARD_WIDTH]; BOARD_HEIGHT];

#[derive(PartialEq)]
enum Player {
    None = 0,
    One = 1,
    Two = 2,
}

enum PlayerInput {
    Column(char),
    Quit,
    Invalid,
}

fn display_board(board: &Board) {
    for board_row in board { 
        // Display each cell in row
        let mut display_row = String::new();
        
        for cell in board_row.iter() {
            display_row.push_str(&"|".blue().to_string());
            if cell == &0 {
                display_row.push_str(&"o".black().to_string());
            } else if cell == &1 {
                display_row.push_str(&"●".red().to_string());
            } else {
                display_row.push_str(&"●".yellow().to_string());
            }
        }
        // Add final pipe
        display_row.push_str(&"|".blue().to_string());
        println!("{}{}", " ".repeat((40-15)/2), display_row);
    }
    
    // Add letters at bottom
    print!(
        "{}{} {} {} {} {} {} {}\n",
        " ".repeat((40-13)/2),
        "A".yellow().bold(),
        "B".yellow().bold(),
        "C".yellow().bold(),
        "D".yellow().bold(),
        "E".yellow().bold(),
        "F".yellow().bold(),
        "G".yellow().bold()
    );
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
    if first_char >= 'a' && first_char < (b'a' + BOARD_WIDTH as u8) as char {
        PlayerInput::Column(first_char)
    } else if first_char == 'q' {
        PlayerInput::Quit
    } else {
        PlayerInput::Invalid
    }
}

fn add_piece_to_column(board: &mut Board, j: usize, player_1_turn: bool) -> Result<(), Box<dyn Error>> {
    for i in (0..board.iter().len()).rev() {
        if board[i][j] == 0 {
            if player_1_turn {
                board[i][j] = 1;
            } else {
                board[i][j] = 2;
            }
            return Ok(())
        }
    }
    Err(Box::from("No space in this column!"))
}

fn check_for_winner(board: &Board, move_number: &u8) -> Option<Player> {
    for i in 0..BOARD_HEIGHT {
        for j in 0..BOARD_WIDTH {
            let cell = board[i][j];
            if cell == 0 {
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
                while column_index < BOARD_WIDTH as isize && column_index >= 0 && row_index < BOARD_HEIGHT as isize && row_index >= 0 {
                    let new_cell = board[row_index as usize][column_index as usize];
                    if new_cell == cell {
                        num_in_row += 1;
                    } else {
                        break;
                    }
                    if num_in_row == 4 {
                        if cell == 1 {
                            return Some(Player::One);
                        } else {
                            return Some(Player::Two);
                        }
                    }
                    row_index += row_offset;
                    column_index += column_offset;
                }
            }
        }
    }

    // Draw if no winner and board full
    if *move_number == BOARD_HEIGHT as u8 * BOARD_WIDTH as u8 {
        return Some(Player::None);
    }

    None
}

fn main() {
    let mut move_number: u8 = 0;

    // Player 1 starts first
    let mut player_1_turn = true;

    // Create board with elements in each row set to 0
    let mut board: Board = [[0; BOARD_WIDTH]; BOARD_HEIGHT];

    // Start gameplay loop
    loop {
        // Clear terminal
        print!("\x1B[2J\x1B[1;1H");

        // Fancy Title
        println!("{:^40}", "Connect 4".magenta().bold().underline());

        // Show board
        display_board(&board);
        println!("Controls: {} Drop Piece | {} Quit\n", "[A-G]".blue(), "[Q]".blue());
        println!("Current Turn: {}", if player_1_turn { "Player 1".to_string().red().bold() } else { "Player 2".to_string().yellow().bold() });
        // User input loop
        loop {
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
                    if let Err(e) = add_piece_to_column(&mut board, column_index as usize, player_1_turn) {
                        println!("{}", e);
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
        move_number += 1;
        // Check for winner
        if let Some(winner) = check_for_winner(&board, &move_number) {
            // Clear terminal
            print!("\x1B[2J\x1B[1;1H");
            // Draw final board
            display_board(&board);

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
