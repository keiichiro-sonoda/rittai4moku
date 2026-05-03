//! 立体4目並べを題材に、Rust とゲーム解析の考え方を学ぶための実験プログラム。
//!
//! このリポジトリは「最短で強いプログラムを完成させる」ことよりも、
//! 二人零和有限確定完全情報ゲームをどのように状態として表し、
//! どのように探索・評価・保存していくのかを、コードを書きながら理解することを目的にする。

/// 盤面の一辺の長さ。
///
/// 立体4目並べでは、盤面を `4 x 4 x 4` の立方体として扱う。
/// この定数を使うことで、配列型やマス数の計算に直接 `4` を散らばらせずに済む。
const BOARD_SIZE: usize = 4;

/// 盤面全体に存在するマスの数。
///
/// `4 x 4 x 4 = 64` マス。
/// 今後、盤面が満杯かどうかの判定や、探索の深さの上限を考えるときに使う。
const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE * BOARD_SIZE;

/// 立体4目並べの盤面。
///
/// 添字はまだ厳密な意味を固定していないが、現時点では
/// `board[z][y][x]` のように「高さ・行・列」と読む想定にしておく。
/// まずは人間が理解しやすい3次元配列で表し、後で必要に応じて
/// 数値化やビットボード表現を追加する。
type Board = [[[Cell; BOARD_SIZE]; BOARD_SIZE]; BOARD_SIZE];

/// 盤面の1マスの状態。
///
/// `Empty` はまだコマが置かれていないマス、
/// `White` と `Black` はそれぞれ白・黒のコマが置かれているマスを表す。
/// メモにあった `0, 1, 2` のような数値表現は、保存や高速化の段階で
/// 別途変換関数として用意する方針にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    /// コマが置かれていない空きマス。
    Empty,

    /// 白のコマが置かれているマス。
    White,

    /// 黒のコマが置かれているマス。
    Black,
}

/// 次に手を打つプレイヤー。
///
/// 盤面上のコマの種類である `Cell::White` / `Cell::Black` と似ているが、
/// `Player` は「これから行動する主体」を表す。
/// 盤面のマス状態と手番を分けることで、ゲーム状態の意味が読みやすくなる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    /// 白番。
    White,

    /// 黒番。
    Black,
}

impl Player {
    /// このプレイヤーが盤面に置くコマの種類を返す。
    ///
    /// 例えば黒番なら `Cell::Black`、白番なら `Cell::White` になる。
    /// 今後 `GameState::play` のような着手関数を作るとき、
    /// 「手番から置くコマを決める」処理をここにまとめておける。
    pub const fn cell(self) -> Cell {
        match self {
            Self::White => Cell::White,
            Self::Black => Cell::Black,
        }
    }

    /// 手番を交代した後のプレイヤーを返す。
    ///
    /// 二人零和ゲームでは、基本的に一手ごとにプレイヤーが交代する。
    /// この関数を使うと、着手後に `turn` を更新する処理を単純に書ける。
    pub const fn next(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameState {
    /// 現在の盤面。
    board: Board,

    /// 次に手を打つプレイヤー。
    turn: Player,

    /// これまでに打たれた手数。
    ///
    /// 最大でも64手なので `u8` で足りる。
    /// 盤面を毎回数え直さずに満杯判定や探索深さの確認ができる。
    moves_played: u8,
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
}

/// ゲーム開始時の状態を表す定数。
///
/// `GameState::initial()` は「初期状態を生成する関数」、
/// `INITIAL_STATE` は「初期状態そのものを表す名前」として使える。
/// どちらの形も見比べながら、Rust の定数と関数の使い分けを学ぶために残している。
const INITIAL_STATE: GameState = GameState::initial();

/// 現在は初期状態を表示するだけの入口。
///
/// 今後、着手生成・勝敗判定・探索の実験コードをここから呼び出す予定。
fn main() {
    println!("{:?}", INITIAL_STATE);
}

#[cfg(test)]
mod tests {
    use super::*;

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
