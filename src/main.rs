use std::{error::Error, io};

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

// Board setup
// 0 represents no player, 1 represents player 1, and 2 represents player 2
type Board = [[u8; BOARD_WIDTH]; BOARD_HEIGHT];

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
    result.push_str("  A|B|C|D|E|F|G\n");
    result
}

fn try_parse_player_input(player_input: String) -> Option<char> {
    let player_input = player_input.trim().to_lowercase();
    // Valid responses only have 1 char
    if player_input.len() != 1 {
        return None
    }
    // Ensure user input is alphabetical character in correct range
    player_input.chars().next().filter(|&i| i >= 'a' && i < (b'a' + BOARD_WIDTH as u8) as char)
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
fn main() {
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

        player_1_turn = !player_1_turn;
    }
}
