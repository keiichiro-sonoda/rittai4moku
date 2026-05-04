use super::{
    ALL_DIRECTIONS, BOARD_SIZE, CELL_COUNT, Cell, Column, Direction, GameStatus, Player, Position,
};

/// 立体4目並べの盤面。
///
/// 添字は `board[z][y][x]` と読む。
///
/// - `x`: 横方向の位置
/// - `y`: 奥行き方向の位置
/// - `z`: 高さ。`0` が一番下、`3` が一番上
///
/// 実物では上から穴の開いたコマを柱に通して落とすため、同じ `(x, y)` の柱では
/// `z = 0` から順番に埋まる。`z = 0` が空なのに `z = 1` だけ埋まる状態は、
/// 通常のプレイでは発生しない不正な状態として扱う。
pub type Board = [[[Cell; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE];

/// 1手を打った結果。
///
/// `state` は着手後の新しい状態、`placed_at` はその手で実際にコマが入った位置。
/// この2つをまとめて返すことで、「次の状態に進む」ことと
/// 「最後の着手位置を使って勝敗判定する」ことの両方ができる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayResult {
    /// 着手後のゲーム状態。
    pub state: GameState,

    /// 今回の手で実際にコマが置かれた3次元座標。
    pub placed_at: Position,
}

/// 着手できなかった理由。
///
/// Rust では、失敗する可能性がある処理を `Result` で表すことが多い。
/// 今回は「盤面外の柱を指定した」場合と「柱がすでに満杯だった」場合を、
/// プログラムが区別できるようにしている。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayError {
    /// `x` または `y` が `0..4` の範囲外だった。
    OutOfBounds,

    /// 指定した柱の `z = 0..3` がすべて埋まっていた。
    ColumnFull,
}

/// 立体4目並べの「状態」。
///
/// ゲーム解析でいう状態とは、「ここから先のゲーム進行を一意に決めるために
/// 必要な情報のまとまり」を指す。
///
/// この構造体では、少なくとも次の3つを状態として持つ。
///
/// - `board`: どのマスにどちらのコマがあるか
/// - `turn`: 次に手を打つのはどちらか
/// - `moves_played`: これまでに何手打たれたか
///
/// 盤面だけではなく手番も状態に含める点が重要。
/// 同じ盤面でも、次が黒番か白番かで「勝てる状態」かどうかが変わるため。
///
/// なお、現在のルールでは `turn` と `moves_played` は盤面から復元できる。
/// 先手は黒で、白黒が必ず交互に打ち、コマが盤面から消えないため。
/// それでも学習段階では、状態遷移を読みやすくするために明示的に持つ。
/// 将来、状態の数値化や完全解析で冗長さが問題になったら、
/// `board` だけを真実として、手番や手数を計算する設計に見直す可能性がある。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameState {
    /// 現在の盤面。
    pub board: Board,

    /// 次に手を打つプレイヤー。
    ///
    /// 盤面上のコマ数から復元できる情報だが、`play` の処理を読みやすくするために持つ。
    pub turn: Player,

    /// これまでに打たれた手数。
    ///
    /// 最大でも64手なので `u8` で足りる。
    /// 盤面を毎回数え直さずに満杯判定や探索深さの確認ができる。
    /// これも盤面から復元できる冗長情報だが、学習段階では明示的に持つ。
    pub moves_played: u8,
}

impl GameState {
    /// ゲーム開始時の初期状態を生成する。
    ///
    /// すべてのマスが空で、手数は0、先手は黒としている。
    /// `const fn` にしているため、実行時に作るだけでなく
    /// `INITIAL_STATE` のような定数の初期化にも使える。
    pub const fn initial() -> Self {
        Self {
            board: [[[Cell::Empty; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE],
            turn: Player::Black,
            moves_played: 0,
        }
    }

    /// 盤面がすべて埋まっているかを返す。
    ///
    /// 立体4目並べでは最大64手で盤面が埋まる。
    /// 勝敗判定をまだ実装していない段階でも、引き分けや探索終了条件を考える
    /// 最初の足場になる。
    pub const fn is_full(&self) -> bool {
        self.moves_played as usize == CELL_COUNT
    }

    /// 指定した柱に次のコマが入る高さ `z` を返す。
    ///
    /// 例えば、ある柱 `(x, y)` がまだ空なら `Some(0)` を返す。
    /// 一番下だけ埋まっていれば次は `Some(1)` になる。
    /// `z = 0, 1, 2, 3` がすべて埋まっている柱なら、もう置けないので `None` を返す。
    pub fn next_empty_z(&self, column: Column) -> Option<usize> {
        if !column.is_in_bounds() {
            return None;
        }

        (0..BOARD_SIZE).find(|&z| self.board[z][column.y][column.x] == Cell::Empty)
    }

    /// 指定した柱が満杯かどうかを返す。
    ///
    /// 重力ありルールでは、柱の一番上 `z = 3` まで埋まると、
    /// その `(x, y)` は合法手ではなくなる。
    pub fn is_column_full(&self, column: Column) -> bool {
        self.next_empty_z(column).is_none()
    }

    /// 現在の状態から選べる合法手を列挙する。
    ///
    /// 重力なしなら空きマスの数だけ合法手があるが、重力ありでは
    /// 「まだ満杯ではない柱」の数だけ合法手がある。
    /// 初期状態では `4 x 4 = 16` 通りになる。
    pub fn legal_moves(&self) -> Vec<Column> {
        let mut moves = Vec::new();

        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let column = Column::new(x, y);
                if !self.is_column_full(column) {
                    moves.push(column);
                }
            }
        }

        moves
    }

    /// 指定した3次元座標にあるマスの状態を返す。
    ///
    /// `Position` は盤面上の実際の場所を表すので、
    /// `board[z][y][x]` の順番で配列にアクセスする。
    /// 今後の勝敗判定では、最後に置かれた場所や、その周囲のマスを見るために使う。
    pub fn cell_at(&self, position: Position) -> Cell {
        self.board[position.z][position.y][position.x]
    }

    /// 指定した位置の隣から、指定方向へ同じマス状態が何個続くかを数える。
    ///
    /// `start` 自身は数えない。`direction` に1歩進んだ場所から数え始める。
    /// 盤面外に出た場合、または違うマス状態に当たった場合にそこで止まる。
    ///
    /// 勝敗判定では、最後に置かれたコマを中心として、
    /// ある方向とその逆方向の両方を数え、最後に置かれたコマ自身の1個を足す。
    pub fn count_same_cells(&self, start: Position, direction: Direction, target: Cell) -> usize {
        let mut count = 0;
        let mut current = start;

        while let Some(next) = direction.step_from(current) {
            if self.cell_at(next) != target {
                break;
            }

            count += 1;
            current = next;
        }

        count
    }

    /// 起点を中心に、指定方向の直線上で同じマス状態が何個つながっているかを数える。
    ///
    /// `count_same_cells` は片方向だけを数えるが、この関数は次の3つを足す。
    ///
    /// - `direction` 方向に続く個数
    /// - `direction.opposite()` 方向に続く個数
    /// - 起点 `start` 自身の1個
    ///
    /// 例えば、横方向に `黒 黒 黒 黒` と並んでいて、起点が内側の黒なら、
    /// 左右を合算して4個と数えられる。
    pub fn count_line_cells(&self, start: Position, direction: Direction, target: Cell) -> usize {
        self.count_same_cells(start, direction, target)
            + self.count_same_cells(start, direction.opposite(), target)
            + 1
    }

    /// 指定した位置のコマが、指定方向で4つ以上つながっているかを返す。
    ///
    /// 空きマスは勝ち判定の起点にならないので、`Cell::Empty` の場合は `false` を返す。
    /// 立体4目並べでは4個並べば勝ちなので、`BOARD_SIZE` 個以上つながっていれば勝ちとする。
    pub fn is_winning_line(&self, start: Position, direction: Direction) -> bool {
        let target = self.cell_at(start);

        target != Cell::Empty && self.count_line_cells(start, direction, target) >= BOARD_SIZE
    }

    /// 指定した位置のコマによって勝ちが成立しているかを返す。
    ///
    /// 最後に置かれた `Position` を渡して使う想定。
    /// 代表13方向をすべて調べ、どれか1方向でも4つ以上つながっていれば勝ちになる。
    pub fn is_winning_position(&self, start: Position) -> bool {
        let target = self.cell_at(start);

        if target == Cell::Empty {
            return false;
        }

        ALL_DIRECTIONS
            .iter()
            .any(|&direction| self.count_line_cells(start, direction, target) >= BOARD_SIZE)
    }

    /// 最後に置かれた位置をもとに、ゲーム全体の進行状態を返す。
    ///
    /// 勝敗は最後に置かれたコマによってだけ新しく発生する。
    /// そのため、毎回すべての盤面を調べるのではなく、
    /// `placed_at` を通るラインで勝ちがあるかを先に見る。
    ///
    /// 判定の順番:
    ///
    /// 1. 最後のコマで勝ちが成立していれば `GameStatus::Win`
    /// 2. 勝ちがなく、盤面が満杯なら `GameStatus::Draw`
    /// 3. それ以外なら `GameStatus::InProgress`
    pub fn status_after_move(&self, placed_at: Position) -> GameStatus {
        if self.is_winning_position(placed_at) {
            let winner = self
                .cell_at(placed_at)
                .player()
                .expect("winning position must contain a player's cell");
            return GameStatus::Win(winner);
        }

        if self.is_full() {
            GameStatus::Draw
        } else {
            GameStatus::InProgress
        }
    }

    /// 指定した柱に現在の手番のコマを落とし、着手結果を返す。
    ///
    /// この関数は元の `GameState` を直接書き換えない。
    /// 代わりに、コピーした状態にコマを置いて返す。
    /// 学習段階では「この状態からこの手を打つと、別の状態ができる」と読む方が、
    /// ゲーム木の考え方と対応しやすい。
    ///
    /// 戻り値には、次の状態だけでなく `placed_at` も含める。
    /// `placed_at` は「最後に置かれた xyz」なので、次の段階で勝敗判定を実装するときに
    /// その場所を通るラインだけを調べる入口になる。
    pub fn play(&self, column: Column) -> Result<PlayResult, PlayError> {
        if !column.is_in_bounds() {
            return Err(PlayError::OutOfBounds);
        }

        let Some(z) = self.next_empty_z(column) else {
            return Err(PlayError::ColumnFull);
        };

        let mut next_state = *self;
        next_state.board[z][column.y][column.x] = self.turn.cell();
        next_state.turn = self.turn.next();
        next_state.moves_played += 1;

        Ok(PlayResult {
            state: next_state,
            placed_at: Position::new(column.x, column.y, z),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{COLUMN_COUNT, Direction, GameStatus};

    /// 初期状態が「空の盤面・黒番・0手目」になっていることを確認する。
    ///
    /// テストは完成品の品質保証だけでなく、
    /// 自分が考えているゲーム状態の定義がコード上でも成り立っているかを
    /// 確認するための学習用メモとしても使う。
    #[test]
    fn initial_state_has_empty_board() {
        let state = GameState::initial();

        assert_eq!(state.turn, Player::Black);
        assert_eq!(state.turn.cell(), Cell::Black);
        assert_eq!(state.turn.next(), Player::White);
        assert_eq!(Cell::Black.player(), Some(Player::Black));
        assert_eq!(Cell::White.player(), Some(Player::White));
        assert_eq!(Cell::Empty.player(), None);
        assert_eq!(state.moves_played, 0);
        assert!(!state.is_full());
        assert_eq!(state.legal_moves().len(), COLUMN_COUNT);
        assert!(
            state
                .board
                .iter()
                .flatten()
                .flatten()
                .all(|cell| *cell == Cell::Empty)
        );
    }

    /// 初期状態では、どの柱も一番下の `z = 0` にコマが入る。
    #[test]
    fn initial_state_drops_piece_to_bottom() {
        let state = GameState::initial();
        let column = Column::new(2, 1);

        assert_eq!(state.next_empty_z(column), Some(0));

        let result = state.play(column).unwrap();

        assert_eq!(result.placed_at, Position::new(2, 1, 0));
        assert_eq!(result.state.board[0][1][2], Cell::Black);
        assert_eq!(result.state.turn, Player::White);
        assert_eq!(result.state.moves_played, 1);
    }

    /// 同じ柱に続けて置くと、コマは `z = 0` から順番に積み上がる。
    ///
    /// `z = 0` が空なのに `z = 1` に置かれる、という状態にはならない。
    #[test]
    fn pieces_stack_in_the_same_column() {
        let column = Column::new(0, 0);
        let first_result = GameState::initial().play(column).unwrap();
        let second_result = first_result.state.play(column).unwrap();
        let state = second_result.state;

        assert_eq!(first_result.placed_at, Position::new(0, 0, 0));
        assert_eq!(second_result.placed_at, Position::new(0, 0, 1));
        assert_eq!(state.board[0][0][0], Cell::Black);
        assert_eq!(state.board[1][0][0], Cell::White);
        assert_eq!(state.board[2][0][0], Cell::Empty);
        assert_eq!(state.next_empty_z(column), Some(2));
    }

    /// 4段すべて埋まった柱は、それ以上合法手として選べない。
    #[test]
    fn full_column_is_not_a_legal_move() {
        let full_column = Column::new(3, 3);
        let mut state = GameState::initial();

        for _ in 0..BOARD_SIZE {
            state = state.play(full_column).unwrap().state;
        }

        assert_eq!(state.next_empty_z(full_column), None);
        assert_eq!(state.play(full_column), Err(PlayError::ColumnFull));
        assert_eq!(state.legal_moves().len(), COLUMN_COUNT - 1);
        assert!(!state.legal_moves().contains(&full_column));
    }

    /// 盤面外の柱は、合法手ではなくエラーになる。
    #[test]
    fn out_of_bounds_column_is_rejected() {
        let state = GameState::initial();

        assert_eq!(
            state.play(Column::new(BOARD_SIZE, 0)),
            Err(PlayError::OutOfBounds)
        );
    }

    /// `cell_at` は、`Position` で指定した `board[z][y][x]` の中身を返す。
    #[test]
    fn cell_at_returns_cell_at_position() {
        let result = GameState::initial().play(Column::new(2, 1)).unwrap();

        assert_eq!(result.state.cell_at(Position::new(2, 1, 0)), Cell::Black);
        assert_eq!(result.state.cell_at(Position::new(2, 1, 1)), Cell::Empty);
    }

    /// 指定方向に同じ色が続いている間だけ数える。
    #[test]
    fn count_same_cells_counts_matching_cells_in_direction() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;

        assert_eq!(
            state.count_same_cells(Position::new(0, 0, 0), Direction::new(1, 0, 0), Cell::Black),
            2
        );
    }

    /// 違う色のコマに当たったら、そこで数えるのを止める。
    #[test]
    fn count_same_cells_stops_at_different_cell() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;

        assert_eq!(
            state.count_same_cells(Position::new(0, 0, 0), Direction::new(1, 0, 0), Cell::Black),
            0
        );
    }

    /// 盤面外に出たら、そこで数えるのを止める。
    #[test]
    fn count_same_cells_stops_at_board_edge() {
        let state = GameState::initial().play(Column::new(0, 0)).unwrap().state;

        assert_eq!(
            state.count_same_cells(
                Position::new(0, 0, 0),
                Direction::new(-1, 0, 0),
                Cell::Black
            ),
            0
        );
    }

    /// 正方向・逆方向・起点自身を合計して、直線上の同じ色の数を数える。
    #[test]
    fn count_line_cells_counts_both_directions_and_start() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;
        state = state.play(Column::new(2, 1)).unwrap().state;
        state = state.play(Column::new(3, 0)).unwrap().state;

        assert_eq!(
            state.count_line_cells(Position::new(1, 0, 0), Direction::new(1, 0, 0), Cell::Black),
            4
        );
    }

    /// 4つ以上つながっている方向があれば、その位置は勝ちになる。
    #[test]
    fn is_winning_position_returns_true_for_four_in_a_row() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;
        state = state.play(Column::new(2, 1)).unwrap().state;
        let result = state.play(Column::new(3, 0)).unwrap();

        assert!(result.state.is_winning_position(result.placed_at));
    }

    /// 4つつながっていない場合は勝ちにならない。
    #[test]
    fn is_winning_position_returns_false_without_four_in_a_row() {
        let result = GameState::initial().play(Column::new(0, 0)).unwrap();

        assert!(!result.state.is_winning_position(result.placed_at));
    }

    /// 空きマスは勝ち判定の起点にならない。
    #[test]
    fn is_winning_position_returns_false_for_empty_cell() {
        let state = GameState::initial();

        assert!(!state.is_winning_position(Position::new(0, 0, 0)));
    }

    /// 最後の手で4つ並んだ場合、ゲーム状態はそのプレイヤーの勝ちになる。
    #[test]
    fn status_after_move_returns_win_for_winning_move() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;
        state = state.play(Column::new(2, 1)).unwrap().state;
        let result = state.play(Column::new(3, 0)).unwrap();

        assert_eq!(
            result.state.status_after_move(result.placed_at),
            GameStatus::Win(Player::Black)
        );
    }

    /// 勝ちがなく、まだ空きがある場合は進行中になる。
    #[test]
    fn status_after_move_returns_in_progress_without_win_or_full_board() {
        let result = GameState::initial().play(Column::new(0, 0)).unwrap();

        assert_eq!(
            result.state.status_after_move(result.placed_at),
            GameStatus::InProgress
        );
    }

    /// 最後に置いた位置で勝ちがなく、盤面が満杯の場合は引き分けになる。
    #[test]
    fn status_after_move_returns_draw_for_full_board_when_last_move_does_not_win() {
        let mut board = [[[Cell::White; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE];
        let placed_at = Position::new(0, 0, 0);
        board[placed_at.z][placed_at.y][placed_at.x] = Cell::Black;

        let state = GameState {
            board,
            turn: Player::Black,
            moves_played: CELL_COUNT as u8,
        };

        assert!(!state.is_winning_position(placed_at));
        assert_eq!(state.status_after_move(placed_at), GameStatus::Draw);
    }
}
