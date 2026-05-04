use crate::game::{GameState, Position};

use super::{MemoTable, Outcome};

/// 直前の着手位置が分かっている局面を解く。
///
/// `GameState::status_after_move` は、最後に置かれた `Position` を使って
/// 勝ち・引き分け・進行中を判定する。
/// そのため、1手以上進んだ局面では `state` だけでなく `placed_at` も渡す。
///
/// 戻り値の `Outcome` は、`state.turn`、つまり次に手を打つプレイヤーから見た結果。
/// 例えば黒が直前の手で勝った局面では、次の手番は白なので `Outcome::Loss` になる。
pub fn solve_after_move(state: &GameState, placed_at: Position, memo: &mut MemoTable) -> Outcome {
    if let Some(outcome) = memo.lookup(state) {
        return outcome;
    }

    if let Some(outcome) =
        Outcome::from_status_for_player(state.status_after_move(placed_at), state.turn)
    {
        memo.remember(state, outcome);
        return outcome;
    }

    solve(state, memo)
}

/// 現在の局面を、次に手を打つプレイヤーから見た `Outcome` として解く。
///
/// この関数は「この局面では、直前の手による勝敗はまだ成立していない」
/// という前提で使う。
/// 初期状態のように `placed_at` が存在しない局面や、
/// `solve_after_move` で進行中だと分かった局面から呼び出す。
///
/// 判定の流れ:
///
/// 1. すでにメモ済みなら、その結果を返す
/// 2. 合法手を1つずつ試す
/// 3. 子局面は相手番なので、子の `Outcome` を `flip()` して自分視点に戻す
/// 4. 勝てる手が1つでもあれば `Win`
/// 5. 勝ちはないが引き分けにできる手があれば `Draw`
/// 6. すべての手が負けにつながるなら `Loss`
///
/// 注意: 現在の `4 x 4 x 4` 盤面で初期状態から呼ぶと探索空間が非常に大きい。
/// まずは仕組みを理解するための最小実装として置いている。
pub fn solve(state: &GameState, memo: &mut MemoTable) -> Outcome {
    if let Some(outcome) = memo.lookup(state) {
        return outcome;
    }

    let mut can_draw = false;

    for column in state.legal_moves() {
        let result = state
            .play(column)
            .expect("legal_moves should only return playable columns");
        let child_outcome = solve_after_move(&result.state, result.placed_at, memo);
        let outcome_for_current_player = child_outcome.flip();

        match outcome_for_current_player {
            Outcome::Win => {
                memo.remember(state, Outcome::Win);
                return Outcome::Win;
            }
            Outcome::Draw => {
                can_draw = true;
            }
            Outcome::Loss => {}
        }
    }

    let outcome = if can_draw {
        Outcome::Draw
    } else {
        Outcome::Loss
    };
    memo.remember(state, outcome);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Column, GameState};

    /// 黒が直前の手で勝った局面は、次に手番が来る白から見ると負けになる。
    #[test]
    fn solve_after_move_returns_loss_for_player_to_move_after_opponent_wins() {
        let mut state = GameState::initial();
        state = state.play(Column::new(0, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;
        state = state.play(Column::new(2, 1)).unwrap().state;
        let result = state.play(Column::new(3, 0)).unwrap();
        let mut memo = MemoTable::new();

        assert_eq!(
            solve_after_move(&result.state, result.placed_at, &mut memo),
            Outcome::Loss
        );
        assert_eq!(memo.lookup(&result.state), Some(Outcome::Loss));
    }

    /// すでにメモにある局面は、合法手を調べずにその結果を返す。
    #[test]
    fn solve_returns_memoized_outcome_without_searching() {
        let state = GameState::initial();
        let mut memo = MemoTable::new();
        memo.remember(&state, Outcome::Draw);

        assert_eq!(solve(&state, &mut memo), Outcome::Draw);
        assert_eq!(memo.len(), 1);
    }

    /// 現在の手番が1手で勝てるなら、その局面は `Outcome::Win` になる。
    ///
    /// このテストでは、黒が `(0, 0)` に置けば横4つが完成する局面を作る。
    /// `legal_moves` は `(0, 0)` から列挙するため、巨大な全探索に入る前に
    /// 最初の合法手で勝ちを見つけられる。
    #[test]
    fn solve_returns_win_when_current_player_has_immediate_winning_move() {
        let mut state = GameState::initial();
        state = state.play(Column::new(1, 0)).unwrap().state;
        state = state.play(Column::new(0, 1)).unwrap().state;
        state = state.play(Column::new(2, 0)).unwrap().state;
        state = state.play(Column::new(1, 1)).unwrap().state;
        state = state.play(Column::new(3, 0)).unwrap().state;
        state = state.play(Column::new(2, 1)).unwrap().state;
        let mut memo = MemoTable::new();

        assert_eq!(solve(&state, &mut memo), Outcome::Win);
        assert_eq!(memo.lookup(&state), Some(Outcome::Win));
    }
}
