use super::Cell;

/// 次に手を打つプレイヤー。
///
/// 盤面上のコマの種類である `Cell::White` / `Cell::Black` と似ているが、
/// `Player` は「これから行動する主体」を表す。
/// 盤面のマス状態と手番を分けることで、ゲーム状態の意味が読みやすくなる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
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
