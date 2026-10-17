use crate::{Board, Player};

pub struct MiniMaxAI;

impl MiniMaxAI {
  // Minimax function
  // Returns best evaluation and best move
  pub fn minimax(&self, board: &mut Board, depth: usize, maximizing_player: &Player) -> (i32, Option<usize>) {
    if depth == 0 {
      return (board.evaluate(), None)
    }

    let valid_moves = board.get_valid_moves();
    let mut best_move = valid_moves.first().copied();
    
    // Positive is better for Player 1 and negative is better for player 2
    let mut best_eval = if maximizing_player == &Player::One { i32::MIN } else { i32::MAX };
    for col in valid_moves {
      // Move piece
      board.move_piece(col, *maximizing_player);

      // Evaluate board at new position
      let (eval, _) = self.minimax(board, depth - 1, if maximizing_player == &Player::One { &Player::Two } else { &Player::One });

      board.unmove_piece(col);

      if maximizing_player == &Player::One {
        if eval > best_eval {
          best_eval = eval;
          best_move = Some(col);
        }
      } else {
        if eval < best_eval {
          best_eval = eval;
          best_move = Some(col);
        }
      }
    }

    (best_eval, best_move)
  }
}