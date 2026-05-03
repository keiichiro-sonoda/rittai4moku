/// 盤面の一辺の長さ。
///
/// 立体4目並べでは、盤面を `4 x 4 x 4` の立方体として扱う。
/// この定数を使うことで、配列型やマス数の計算に直接 `4` を散らばらせずに済む。
pub const BOARD_SIZE: usize = 4;

/// 盤面全体に存在するマスの数。
///
/// `4 x 4 x 4 = 64` マス。
/// 今後、盤面が満杯かどうかの判定や、探索の深さの上限を考えるときに使う。
pub const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE * BOARD_SIZE;

/// 上からコマを落とせる柱の本数。
///
/// 実物の立体4目並べでは `4 x 4` 本の柱があり、プレイヤーはその柱を1本選ぶ。
/// コマの高さ `z` は自分で選ぶのではなく、重力によって自動的に決まる。
pub const COLUMN_COUNT: usize = BOARD_SIZE * BOARD_SIZE;
