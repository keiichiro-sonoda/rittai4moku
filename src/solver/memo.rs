use std::collections::HashMap;

use crate::game::GameState;

use super::Outcome;

/// 探索済み局面の結果を保存するメモ帳。
///
/// メモ化とは、一度計算した結果を保存しておき、
/// 同じ入力が再び出てきたときに再計算せず取り出す方法。
///
/// この型では、`GameState::board_key_base3()` が返す盤面キーを `HashMap` のキーにする。
/// 値には、その局面を探索した結果である `Outcome` を保存する。
///
/// まだ完全探索は実装しない。
/// まずは「盤面キーで保存する」「同じ盤面キーで取り出す」という
/// メモ化の入口だけを学習できるようにする。
#[derive(Debug, Default, Clone)]
pub struct MemoTable {
    entries: HashMap<u128, Outcome>,
}

impl MemoTable {
    /// 空のメモ表を作る。
    ///
    /// 最初は何も探索していないため、保存済みの局面は0件。
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// 保存済みの局面数を返す。
    ///
    /// 学習段階では、探索が進むにつれてメモが増えているかを確認するために使う。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// メモが空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 指定した局面の探索結果を保存する。
    ///
    /// `state.board_key_base3()` をキーとして使うため、呼び出し側は
    /// 盤面キーの作り方を毎回意識しなくてよい。
    ///
    /// 同じキーがすでに保存されていた場合は、古い `Outcome` を返す。
    /// 初めて保存する局面なら `None` を返す。
    pub fn remember(&mut self, state: &GameState, outcome: Outcome) -> Option<Outcome> {
        self.entries.insert(state.board_key_base3(), outcome)
    }

    /// 指定した局面の探索結果がメモにあれば返す。
    ///
    /// まだ保存されていない局面なら `None` を返す。
    /// 将来の探索では、`None` のときだけ実際に合法手を調べ、
    /// 結果が分かったら `remember` で保存する流れになる。
    pub fn lookup(&self, state: &GameState) -> Option<Outcome> {
        self.entries.get(&state.board_key_base3()).copied()
    }

    /// 指定した局面がメモ済みかどうかを返す。
    pub fn contains(&self, state: &GameState) -> bool {
        self.entries.contains_key(&state.board_key_base3())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Column, GameState};

    /// 作ったばかりのメモ表には、まだ何も保存されていない。
    #[test]
    fn new_memo_table_is_empty() {
        let memo = MemoTable::new();

        assert_eq!(memo.len(), 0);
        assert!(memo.is_empty());
        assert_eq!(memo.lookup(&GameState::initial()), None);
    }

    /// 局面と結果を保存すると、同じ局面から同じ結果を取り出せる。
    #[test]
    fn remember_then_lookup_returns_saved_outcome() {
        let state = GameState::initial();
        let mut memo = MemoTable::new();

        assert_eq!(memo.remember(&state, Outcome::Draw), None);

        assert_eq!(memo.len(), 1);
        assert!(!memo.is_empty());
        assert!(memo.contains(&state));
        assert_eq!(memo.lookup(&state), Some(Outcome::Draw));
    }

    /// 違う盤面は違うキーになるため、別々のメモとして保存される。
    #[test]
    fn different_states_are_stored_separately() {
        let initial = GameState::initial();
        let after_one_move = initial.play(Column::new(0, 0)).unwrap().state;
        let mut memo = MemoTable::new();

        memo.remember(&initial, Outcome::Draw);
        memo.remember(&after_one_move, Outcome::Win);

        assert_eq!(memo.len(), 2);
        assert_eq!(memo.lookup(&initial), Some(Outcome::Draw));
        assert_eq!(memo.lookup(&after_one_move), Some(Outcome::Win));
    }

    /// 同じ局面をもう一度保存すると、新しい結果で上書きされる。
    ///
    /// `HashMap::insert` と同じく、戻り値には上書き前の値が入る。
    /// 探索では通常同じ局面に矛盾した結果を入れないが、
    /// ここでは `HashMap` の基本的な挙動を学ぶために確認する。
    #[test]
    fn remember_overwrites_existing_outcome_and_returns_old_one() {
        let state = GameState::initial();
        let mut memo = MemoTable::new();

        assert_eq!(memo.remember(&state, Outcome::Draw), None);
        assert_eq!(memo.remember(&state, Outcome::Loss), Some(Outcome::Draw));

        assert_eq!(memo.len(), 1);
        assert_eq!(memo.lookup(&state), Some(Outcome::Loss));
    }
}
