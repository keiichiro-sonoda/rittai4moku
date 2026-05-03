use super::BOARD_SIZE;

/// プレイヤーが選ぶ「柱」。
///
/// 重力ありの立体4目並べでは、プレイヤーは `(x, y, z)` の3次元座標を直接選ばない。
/// 選ぶのは上からコマを落とす柱、つまり `(x, y)` だけ。
/// 実際に入る高さ `z` は、その柱の下から何段目まで埋まっているかで決まる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    /// 横方向の位置。`0..4` の範囲を使う。
    pub x: usize,

    /// 奥行き方向の位置。`0..4` の範囲を使う。
    pub y: usize,
}

impl Column {
    /// 新しい柱座標を作る。
    ///
    /// ここでは範囲チェックをしない。
    /// 範囲外の座標をどう扱うかは、実際に着手するときの `GameState::play` で
    /// `Result` として返す。
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// この柱が `4 x 4` の盤面内にあるかを返す。
    pub const fn is_in_bounds(self) -> bool {
        self.x < BOARD_SIZE && self.y < BOARD_SIZE
    }
}

/// 実際にコマが置かれた3次元座標。
///
/// `Column` が「プレイヤーが選ぶ柱」なのに対して、
/// `Position` は「重力によって最終的にコマが入った場所」を表す。
///
/// 例えば、プレイヤーが `Column { x: 2, y: 1 }` を選んだとき、
/// その柱が空なら、実際の置き場所は `Position { x: 2, y: 1, z: 0 }` になる。
/// 勝敗判定では、この「最後に置かれた場所」から伸びる直線だけを調べればよいので、
/// 毎回すべての勝利ラインを調べるより考えやすくなる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// 横方向の位置。`0..4` の範囲を使う。
    pub x: usize,

    /// 奥行き方向の位置。`0..4` の範囲を使う。
    pub y: usize,

    /// 高さ方向の位置。`0` が一番下、`3` が一番上。
    pub z: usize,
}

impl Position {
    /// 新しい3次元座標を作る。
    pub const fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}
