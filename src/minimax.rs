use crate::{Board, Player, BOARD_COLS};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq)]
enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

struct TTEntry {
    depth: usize,
    eval: i32,
    node_type: NodeType,
    best_move: Option<usize>,
}

pub struct MiniMaxAI {
    table: HashMap<Board, TTEntry>,
}

impl MiniMaxAI {
  pub fn new() -> Self {
    MiniMaxAI {
      table: HashMap::new(),
    }
  }

  // Minimax function
  // Returns best evaluation and best move
  pub fn minimax(&mut self, board: &mut Board, target_depth: usize, maximizing_player: &Player) -> (i32, Option<usize>) {
    self.table.clear();
    let mut best_eval = 0;
    let mut best_move = None;

    // Iterative deepening: starts at low depth and slowly work down to target depth, reusing saved evaluations if possible
    for current_depth in 1..=target_depth {
      println!("current depth: {}", current_depth);
      let (eval, col) = self.minimax_ply(board, current_depth, 0, i32::MIN, i32::MAX, maximizing_player);
      best_eval = eval;
      if col.is_some() {
        best_move = col
      }
      // If found winning move, don't need to evaluate further
      if eval.abs() > 900_000 {
        break;
      }
    }

    (best_eval, best_move)
  }

  fn minimax_ply(&mut self, board: &mut Board, depth: usize, ply: usize, mut alpha: i32, mut beta: i32, maximizing_player: &Player) -> (i32, Option<usize>) {
    let alpha_orig = alpha;
    let beta_orig = beta;

    // If there is already entry in tablebase with higher depth, use its value.
    let mut best_move_hint = None;

    if let Some(entry) = self.table.get(board) {
      best_move_hint = entry.best_move;
      if entry.depth >= depth {
        let mut cached_eval = entry.eval;
        // If cached eval is winning or losing, prefer earlier wins and later losses
        if cached_eval > 900_000 { cached_eval -= (ply as i32) * 100; }
        else if cached_eval < -900_000 { cached_eval += (ply as i32) * 100; }

        match entry.node_type {
          NodeType::Exact => return (cached_eval, entry.best_move),
          NodeType::LowerBound => alpha = alpha.max(cached_eval),
          NodeType::UpperBound => beta = beta.min(cached_eval),
        }
        // Prune branch
        if alpha >= beta { return (cached_eval, entry.best_move); }
      }
    }

    // If reached end, just use current eval
    if depth == 0 {
      return (board.evaluate(), None)
    }

    let mut valid_moves = board.get_valid_moves();
    if valid_moves.is_empty() {
      return (board.evaluate(), None);
    }

    // Prioritize best move from previous depth search
    // Do center moves first bc more efficient for alpha-beta pruning
    valid_moves.sort_by_key(|&col| {
      if Some(col) == best_move_hint {
        return -100;
      }
      let center = (BOARD_COLS / 2) as isize;
      (col as isize - center).abs()
    });

    let mut best_move = valid_moves.first().copied();
    let is_player_one = maximizing_player == &Player::One;
    let mut best_eval = if is_player_one { i32::MIN } else { i32::MAX };

    for col in valid_moves {
      let eval;
      // move_piece returns the row where the piece landed
      let placed_row = board.move_piece(col, *maximizing_player).unwrap();

      // Smaller ply means earlier win or later loss so is better
      if board.check_win_at(placed_row, col) == Some(*maximizing_player) {
        eval = 1_000_000 - (ply as i32) * 100;
      } else if board.check_win_at(placed_row, col) == Some(if is_player_one { Player::Two } else { Player::One }) {
        eval = -1_000_000 + (ply as i32) * 100;
      } else {
        // Recurse with incremented ply and updated alpha/beta bounds
        (eval, _) = self.minimax_ply(board, depth - 1, ply + 1, alpha, beta, if is_player_one { &Player::Two } else { &Player::One });
      }
      board.unmove_piece(col);

      if is_player_one {
        if eval > best_eval {
          best_eval = eval;
          best_move = Some(col);
        }
        // alpha = max eval for player 1
        alpha = alpha.max(best_eval);
        if beta <= alpha {
          break; // Prune branch
        }
      } else {
        if eval < best_eval {
          best_eval = eval;
          best_move = Some(col);
        }
        // beta = min eval for player 2
        beta = beta.min(best_eval);
        if beta <= alpha {
          break; // Prune branch
        }
      }
    }

    let mut store_eval = best_eval;
    if store_eval > 900_000 { store_eval += (ply as i32) * 100; }
    else if store_eval < -900_000 { store_eval -= (ply as i32) * 100; }

    let node_type = if best_eval <= alpha_orig { NodeType::UpperBound } // Eval is new min
                    else if best_eval >= beta_orig { NodeType::LowerBound } // Eval is new max
                    else { NodeType::Exact };

    self.table.insert(board.clone(), TTEntry {
        depth,
        eval: store_eval,
        node_type,
        best_move,
    });

    (best_eval, best_move)
  }
}