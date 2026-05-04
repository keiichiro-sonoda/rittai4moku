use std::collections::HashSet;
use std::env;
use std::time::Instant;

use rittai4moku::game::{GameState, GameStatus};

/// 正規化なしで、手数ごとの到達可能状態数を数える実験プログラム。
///
/// この example は完全解析そのものではなく、
/// 「正規化しないと、どの手数で状態数がどれくらい増えるのか」を測るための道具。
///
/// 重要な前提:
///
/// - 状態は `GameState::board_key_base3()` の `u128` で保存する
/// - 異なる手数の状態は同じ盤面にならないので、過去手数との重複排除はしない
/// - 同じ手数の別経路から同じ盤面に到達する可能性はあるので、同一手数内では重複排除する
/// - 「次の一手で勝てる状態」は葉として扱い、それ以上展開しない
///
/// 使い方:
///
/// ```text
/// cargo run --release --example frontier_counts -- 8
/// ```
///
/// 引数を省略した場合は、8手目まで数える。
fn main() {
    let max_ply = env::args()
        .nth(1)
        .map(|arg| {
            arg.parse::<u8>()
                .expect("max ply must be an integer between 0 and 255")
        })
        .unwrap_or(8);

    let mut frontier = HashSet::from([GameState::initial().board_key_base3()]);

    println!("max_ply: {max_ply}");
    println!(
        "ply,frontier,terminal_immediate_win,terminal_draw,expanded,children_generated,next_unique,duplicate_children,estimated_frontier_key_mib,estimated_next_key_mib,elapsed_ms"
    );

    for ply in 0..=max_ply {
        let started_at = Instant::now();
        let mut next_frontier = HashSet::new();
        let mut terminal_immediate_win = 0_u64;
        let mut terminal_draw = 0_u64;
        let mut expanded = 0_u64;
        let mut children_generated = 0_u64;

        for &key in &frontier {
            let state = GameState::from_board_key_base3(key)
                .expect("frontier keys should come from valid GameState values");

            if state.is_full() {
                terminal_draw += 1;
                continue;
            }

            if has_immediate_winning_move(&state) {
                terminal_immediate_win += 1;
                continue;
            }

            expanded += 1;

            if ply < max_ply {
                for column in state.legal_moves() {
                    let result = state
                        .play(column)
                        .expect("legal_moves should only return playable columns");
                    next_frontier.insert(result.state.board_key_base3());
                    children_generated += 1;
                }
            }
        }

        let duplicate_children = children_generated.saturating_sub(next_frontier.len() as u64);

        println!(
            "{ply},{},{terminal_immediate_win},{terminal_draw},{expanded},{children_generated},{},{duplicate_children},{:.3},{:.3},{}",
            frontier.len(),
            next_frontier.len(),
            estimated_key_mib(frontier.len()),
            estimated_key_mib(next_frontier.len()),
            started_at.elapsed().as_millis(),
        );

        frontier = next_frontier;
    }
}

/// 現在の手番プレイヤーが、次の1手で勝てるかを返す。
///
/// 完全解析では、勝てる手が1つでもある局面は `Outcome::Win` と分類できる。
/// 今回の状態数調査では、その局面を葉として扱い、
/// 「勝てるのに別の手を選んだ先」は展開しない。
fn has_immediate_winning_move(state: &GameState) -> bool {
    state.legal_moves().into_iter().any(|column| {
        let result = state
            .play(column)
            .expect("legal_moves should only return playable columns");
        result.state.status_after_move(result.placed_at) == GameStatus::Win(state.turn)
    })
}

/// `u128` のキー本体だけを保存した場合の概算MiB。
///
/// `HashSet` の実際のメモリ使用量は、ハッシュ表の空きバケットや制御情報も持つため、
/// この値よりかなり大きくなる。
/// ここでは「キーそのものだけで最低これくらい」という目安として表示する。
fn estimated_key_mib(key_count: usize) -> f64 {
    key_count as f64 * size_of::<u128>() as f64 / 1024.0 / 1024.0
}
