const BOARD_SIZE: usize = 4;
const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE * BOARD_SIZE;

type Board = [[[Cell; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    White,
    Black,
}

impl Player {
    pub const fn cell(self) -> Cell {
        match self {
            Self::White => Cell::White,
            Self::Black => Cell::Black,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameState {
    board: Board,
    turn: Player,
    moves_played: u8,
}

impl GameState {
    pub const fn initial() -> Self {
        Self {
            board: [[[Cell::Empty; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE],
            turn: Player::Black,
            moves_played: 0,
        }
    }

    pub const fn is_full(&self) -> bool {
        self.moves_played as usize == CELL_COUNT
    }
}

const INITIAL_STATE: GameState = GameState::initial();

fn main() {
    println!("{:?}", INITIAL_STATE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_has_empty_board() {
        let state = GameState::initial();

        assert_eq!(state.turn, Player::Black);
        assert_eq!(state.turn.cell(), Cell::Black);
        assert_eq!(state.turn.next(), Player::White);
        assert_eq!(state.moves_played, 0);
        assert!(!state.is_full());
        assert!(
            state
                .board
                .iter()
                .flatten()
                .flatten()
                .all(|cell| *cell == Cell::Empty)
        );
    }
}
