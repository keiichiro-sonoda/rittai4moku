use super::Player;

/// ゲーム全体の進行状態。
///
/// `GameState` は盤面・手番・手数を持つ「状態」そのものを表す。
/// `GameStatus` は、その状態がゲームとしてどう評価されるかを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    /// まだ勝敗が決まっておらず、次の手を打てる状態。
    InProgress,

    /// 指定したプレイヤーが勝った状態。
    Win(Player),

    /// 盤面がすべて埋まり、勝者がいない状態。
    Draw,
}
