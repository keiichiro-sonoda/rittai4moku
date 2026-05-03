use super::{BOARD_SIZE, Position};

/// 3次元盤面上で直線を調べるための方向。
///
/// `dx`, `dy`, `dz` は、それぞれ `x`, `y`, `z` を何マス進めるかを表す。
/// 例えば `Direction::new(1, 0, 0)` は横方向に1マス進む方向、
/// `Direction::new(0, 0, 1)` は高さ方向に1マス上がる方向になる。
///
/// 勝敗判定では、最後に置かれた `Position` からこの方向と逆方向をたどり、
/// 同じ色のコマが合計4つ以上つながっているかを調べる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Direction {
    /// `x` 方向に進む量。
    pub dx: isize,

    /// `y` 方向に進む量。
    pub dy: isize,

    /// `z` 方向に進む量。
    pub dz: isize,
}

impl Direction {
    /// 新しい方向を作る。
    pub const fn new(dx: isize, dy: isize, dz: isize) -> Self {
        Self { dx, dy, dz }
    }

    /// 逆向きの方向を返す。
    ///
    /// 例えば `(1, 0, 0)` の逆は `(-1, 0, 0)`。
    /// 勝敗判定では、最後に置かれたコマから正方向と逆方向の両方を数える。
    pub const fn opposite(self) -> Self {
        Self {
            dx: -self.dx,
            dy: -self.dy,
            dz: -self.dz,
        }
    }

    /// 指定した座標から、この方向に1マス進んだ座標を返す。
    ///
    /// 進んだ先が盤面の外なら `None` を返す。
    /// `Position` の各値は `usize` だが、方向には `-1` が出てくるため、
    /// いったん `isize` で計算してから盤面内かどうかを確認する。
    pub fn step_from(self, position: Position) -> Option<Position> {
        let x = position.x as isize + self.dx;
        let y = position.y as isize + self.dy;
        let z = position.z as isize + self.dz;

        if is_in_bounds(x) && is_in_bounds(y) && is_in_bounds(z) {
            Some(Position::new(x as usize, y as usize, z as usize))
        } else {
            None
        }
    }
}

/// 勝敗判定で調べる代表方向。
///
/// 3次元では、各軸について `-1, 0, 1` の進み方があり、何も動かない `(0, 0, 0)` を
/// 除くと26方向ある。ただし、直線は正方向と逆方向をセットで調べればよいので、
/// 代表方向は半分の13方向で足りる。
///
/// 方向の内訳:
///
/// - 3方向: x, y, z の軸方向
/// - 6方向: xy, xz, yz 平面上の斜め
/// - 4方向: 立方体を貫く空間斜め
pub const ALL_DIRECTIONS: [Direction; 13] = [
    Direction::new(1, 0, 0),
    Direction::new(0, 1, 0),
    Direction::new(0, 0, 1),
    Direction::new(1, 1, 0),
    Direction::new(1, -1, 0),
    Direction::new(1, 0, 1),
    Direction::new(1, 0, -1),
    Direction::new(0, 1, 1),
    Direction::new(0, 1, -1),
    Direction::new(1, 1, 1),
    Direction::new(1, 1, -1),
    Direction::new(1, -1, 1),
    Direction::new(1, -1, -1),
];

fn is_in_bounds(value: isize) -> bool {
    0 <= value && value < BOARD_SIZE as isize
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 3次元盤面で、正逆をまとめて調べる代表方向は13個。
    #[test]
    fn all_directions_has_13_representatives() {
        assert_eq!(ALL_DIRECTIONS.len(), 13);
    }

    /// 盤面内に進める場合は、進んだ先の `Position` が返る。
    #[test]
    fn step_from_returns_next_position_inside_board() {
        let position = Position::new(1, 1, 1);
        let direction = Direction::new(1, -1, 1);

        assert_eq!(direction.step_from(position), Some(Position::new(2, 0, 2)));
    }

    /// 盤面外に出る場合は `None` になる。
    #[test]
    fn step_from_returns_none_outside_board() {
        let position = Position::new(0, 0, 0);
        let direction = Direction::new(-1, 0, 0);

        assert_eq!(direction.step_from(position), None);
    }

    /// 逆方向は、各成分の符号を反転した方向になる。
    #[test]
    fn opposite_reverses_direction() {
        let direction = Direction::new(1, -1, 0);

        assert_eq!(direction.opposite(), Direction::new(-1, 1, 0));
    }
}
