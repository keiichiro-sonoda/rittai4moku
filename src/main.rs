use rittai4moku::game::INITIAL_STATE;

/// 現在は初期状態を表示するだけの入口。
///
/// ゲームのルールや状態表現は `game` モジュール側に置く。
/// `main.rs` は、今後コマンドライン実行や実験コードを呼び出す場所として使う。
fn main() {
    println!("{:?}", INITIAL_STATE);
}
