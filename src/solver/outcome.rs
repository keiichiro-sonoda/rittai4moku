use crate::game::{GameStatus, Player};

/// 探索で得られる局面の結果。
///
/// `GameStatus` は「ゲームが進行中か、誰が勝ったか」という
/// ゲーム全体の進行ラベルを表す。
/// 一方で `Outcome` は、探索中のある `GameState` について、
/// 「次に手を打つプレイヤーから見て」勝ち・負け・引き分けのどれかを表す。
///
/// 例えば同じ黒勝ちの終局でも、その局面で黒番として評価するのか、
/// 白番として評価するのかで `Win` / `Loss` の見方が変わる。
/// ミニマックス探索では、この「手番側から見た結果」として持つ方が扱いやすい。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 次に手を打つプレイヤーが、正しく進めれば勝てる局面。
    Win,

    /// 次に手を打つプレイヤーが、相手に正しく応じられると負ける局面。
    Loss,

    /// どちらも勝ちを強制できず、引き分けになる局面。
    Draw,
}

impl Outcome {
    /// 評価する視点を相手側へ反転した結果を返す。
    ///
    /// 再帰探索では、自分が1手打つと次は相手番になる。
    /// そのため、子局面を調べて得られる `Outcome` は「相手から見た結果」になる。
    /// それを親局面の自分から見た結果へ戻すために、この関数を使う。
    ///
    /// - 相手から見て `Win` なら、自分から見れば `Loss`
    /// - 相手から見て `Loss` なら、自分から見れば `Win`
    /// - `Draw` はどちらから見ても `Draw`
    pub const fn flip(self) -> Self {
        match self {
            Self::Win => Self::Loss,
            Self::Loss => Self::Win,
            Self::Draw => Self::Draw,
        }
    }

    /// `GameStatus` を、指定したプレイヤーから見た `Outcome` に変換する。
    ///
    /// `GameStatus` は「ゲームが進行中か」「誰が勝ったか」を表す。
    /// それに対して `Outcome` は「あるプレイヤーから見て勝ちか負けか」を表す。
    /// そのため、変換には `perspective`、つまり評価するプレイヤーが必要になる。
    ///
    /// - `GameStatus::Win(winner)` で `winner == perspective` なら `Some(Outcome::Win)`
    /// - `GameStatus::Win(winner)` で `winner != perspective` なら `Some(Outcome::Loss)`
    /// - `GameStatus::Draw` なら、誰から見ても `Some(Outcome::Draw)`
    /// - `GameStatus::InProgress` はまだ結果が決まっていないため `None`
    ///
    /// 将来の探索では、終局状態だけをこの関数で `Outcome` に変換し、
    /// 進行中の状態は合法手を再帰的に調べて `Outcome` を決める。
    pub fn from_status_for_player(status: GameStatus, perspective: Player) -> Option<Self> {
        match status {
            GameStatus::InProgress => None,
            GameStatus::Draw => Some(Self::Draw),
            GameStatus::Win(winner) => {
                if winner == perspective {
                    Some(Self::Win)
                } else {
                    Some(Self::Loss)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 勝者本人から見れば、勝ちの終局状態は `Outcome::Win` になる。
    #[test]
    fn win_status_is_win_from_winners_perspective() {
        assert_eq!(
            Outcome::from_status_for_player(GameStatus::Win(Player::Black), Player::Black),
            Some(Outcome::Win)
        );
    }

    /// 勝者ではない側から見れば、勝ちの終局状態は `Outcome::Loss` になる。
    #[test]
    fn win_status_is_loss_from_losers_perspective() {
        assert_eq!(
            Outcome::from_status_for_player(GameStatus::Win(Player::Black), Player::White),
            Some(Outcome::Loss)
        );
    }

    /// 引き分けは、どちらのプレイヤーから見ても `Outcome::Draw` になる。
    #[test]
    fn draw_status_is_draw_from_any_perspective() {
        assert_eq!(
            Outcome::from_status_for_player(GameStatus::Draw, Player::Black),
            Some(Outcome::Draw)
        );
        assert_eq!(
            Outcome::from_status_for_player(GameStatus::Draw, Player::White),
            Some(Outcome::Draw)
        );
    }

    /// 進行中の状態はまだ探索結果ではないため、`Outcome` には変換しない。
    #[test]
    fn in_progress_status_has_no_outcome_yet() {
        assert_eq!(
            Outcome::from_status_for_player(GameStatus::InProgress, Player::Black),
            None
        );
    }

    /// 視点を反転すると、勝ちは負けに、負けは勝ちになる。
    #[test]
    fn flip_swaps_win_and_loss() {
        assert_eq!(Outcome::Win.flip(), Outcome::Loss);
        assert_eq!(Outcome::Loss.flip(), Outcome::Win);
    }

    /// 引き分けは、どちらの視点から見ても引き分けのまま。
    #[test]
    fn flip_keeps_draw_as_draw() {
        assert_eq!(Outcome::Draw.flip(), Outcome::Draw);
    }

    /// 2回反転すると、元の視点に戻る。
    #[test]
    fn flipping_twice_returns_original_outcome() {
        for outcome in [Outcome::Win, Outcome::Loss, Outcome::Draw] {
            assert_eq!(outcome.flip().flip(), outcome);
        }
    }
}
