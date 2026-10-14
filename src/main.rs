use std::{error::Error, io};

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


fn display_board(board: &Board) -> String {
    let mut result = String::new();
    
    for row in board {
        // Display each cell in row
        for (j, cell) in row.iter().enumerate() {
            result.push_str(&cell.to_string());
            // Don't add pipe after last cell
            if j < BOARD_WIDTH - 1 {
                result.push('|');
            }
        }
        result.push('\n');
    }
    // Add letters at bottom
    result.push_str("A|B|C|D|E|F|G\n");
    result
}

fn try_parse_player_input(player_input: String) -> Option<char> {
    let player_input = player_input.trim().to_lowercase();
    let mut chars = player_input.chars();
    let first_char = chars.next()?;
    // Valid responses only have 1 char
    // If chars has multiple characters, return None
    if chars.next().is_some() {
        return None
    }
    // Ensure user input is alphabetical character in correct range
    if first_char >= 'a' && first_char < (b'a' + BOARD_WIDTH as u8) as char {
        Some(first_char)
    } else {
        None
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
        println!("{}", display_board(&board));
        println!("Player to move: {}", if player_1_turn {1} else {2});
        // User input loop
        loop {
            println!("Enter your move as a letter: ");
            let mut player_input = String::new();
            io::stdin()
            .read_line(&mut player_input)
            .expect("Failed to read user input");
            if let Some(c) = try_parse_player_input(player_input) {
                let column_index = (c as u8) - ('a' as u8);
                if let Err(e) = add_piece_to_column(&mut board, column_index as usize, player_1_turn) {
                    println!("{}", e);
                    continue; // Ask for input again
                }
                break;
            } else {
                continue;
            }
        }
        move_number += 1;
        // Check for winner
        println!("Checking for winner");
        if let Some(winner) = check_for_winner(&board, &move_number) {
            // Draw final board
            println!("{}", display_board(&board));

            if winner == Player::One {
                println!("Player 1 won!");
            } else if winner == Player::Two {
                println!("Player 2 won!");
            } else {
                println!("It was a draw!");
            }
            break;
        }

        player_1_turn = !player_1_turn;
    }
}
