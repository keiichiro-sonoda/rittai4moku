use super::Player;

/// 盤面の1マスの状態。
///
/// `Empty` はまだコマが置かれていないマス、
/// `White` と `Black` はそれぞれ白・黒のコマが置かれているマスを表す。
/// メモにあった `0, 1, 2` のような数値表現は、保存や高速化の段階で
/// 別途変換関数として用意する方針にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    /// コマが置かれていない空きマス。
    Empty,

    /// 白のコマが置かれているマス。
    White,

    /// 黒のコマが置かれているマス。
    Black,
}

impl Cell {
    /// 盤面を3進数として数値化するときの桁の値を返す。
    ///
    /// 1マスは「空・白・黒」の3状態なので、3進数の1桁として表せる。
    ///
    /// - `Cell::Empty` は `0`
    /// - `Cell::Black` は `1`
    /// - `Cell::White` は `2`
    ///
    /// 黒を1、白を2にしているのは、先手の黒を先に扱うと読みやすいため。
    pub const fn base3_digit(self) -> u128 {
        match self {
            Self::Empty => 0,
            Self::Black => 1,
            Self::White => 2,
        }
    }

    /// 3進数の1桁から `Cell` に戻す。
    ///
    /// `base3_digit` の逆変換として使う。
    ///
    /// - `0` は `Some(Cell::Empty)`
    /// - `1` は `Some(Cell::Black)`
    /// - `2` は `Some(Cell::White)`
    /// - それ以外は3進数の桁として不正なので `None`
    pub const fn from_base3_digit(digit: u128) -> Option<Self> {
        match digit {
            0 => Some(Self::Empty),
            1 => Some(Self::Black),
            2 => Some(Self::White),
            _ => None,
        }
    }

    /// このマスに置かれているコマのプレイヤーを返す。
    ///
    /// `Cell::White` なら `Some(Player::White)`、
    /// `Cell::Black` なら `Some(Player::Black)` になる。
    /// `Cell::Empty` は誰のコマでもないので `None` を返す。
    pub const fn player(self) -> Option<Player> {
        match self {
            Self::Empty => None,
            Self::White => Some(Player::White),
            Self::Black => Some(Player::Black),
        }
    }
}
