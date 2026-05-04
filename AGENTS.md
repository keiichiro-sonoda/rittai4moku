# AGENTS.md

AI コーディングエージェント向けの規約。人間向けの概要は README.md、設計判断の根拠は docs/adr/ にある。本ファイルでは規約と索引だけを扱う。

## プロジェクト

立体4目並べ（4×4×4・重力あり）の状態表現と探索を Rust で書いて学ぶ実験リポジトリ。最終目標は完全解析。現段階はライブラリ層の整備中で、CLI・探索・評価関数は未実装。

## コマンド

- ビルド: `cargo build`
- テスト: `cargo test`
- 整形: `cargo fmt --all`
- 静的解析: `cargo clippy --all-targets -- -D warnings`
- 状態数調査: `cargo run --example frontier_counts -- 8`

外部クレートの追加は事前に相談する。

## ディレクトリ

- `src/lib.rs`: ライブラリ入口。`game` を `pub mod` で公開。
- `src/main.rs`: 実験用エントリ。現状は `INITIAL_STATE` を表示するのみ。
- `src/game/mod.rs`: モジュール集約と再エクスポート、`INITIAL_STATE` 定義。
- `src/game/cell.rs`: `Cell { Empty, Black, White }` と 3進数桁との変換。
- `src/game/player.rs`: `Player { Black, White }`、`cell()` / `next()`。
- `src/game/coordinate.rs`: `Column(x, y)`、`Position(x, y, z)`。
- `src/game/line.rs`: `Direction` と代表13方向 `ALL_DIRECTIONS`。
- `src/game/constants.rs`: `BOARD_SIZE = 4`, `CELL_COUNT = 64`, `COLUMN_COUNT = 16`。
- `src/game/state.rs`: `GameState`, `Board`, `play`, 勝敗判定, 盤面キー往復。
- `src/game/status.rs`: `GameStatus { InProgress, Win(Player), Draw }`。
- `src/solver/mod.rs`: 探索用モジュールの集約。
- `src/solver/outcome.rs`: 手番側から見た探索結果 `Outcome { Win, Loss, Draw }`、視点反転、`GameStatus` からの変換。
- `src/solver/memo.rs`: `HashMap<u128, Outcome>` を包むメモ化用 `MemoTable`。
- `src/solver/search.rs`: 再帰探索の最小実装 `solve` / `solve_after_move`。
- `examples/frontier_counts.rs`: 正規化なしで手数ごとの一意な盤面キー数を数える実験。過去手数とは重複排除しない。
- `docs/adr/`: 設計判断の記録。
- `examples/`, `tests/`, `benches/`: 用意でき次第使う（今は未作成）。

## 用語

- `Cell`: 1マスの状態（Empty/Black/White）。
- `Player`: 手番の主体（Black/White）。先手は Black。
- `Column(x, y)`: プレイヤーが選ぶ柱。`z` は重力で決まる。
- `Position(x, y, z)`: 実際にコマが入った3次元座標。
- `Direction`: 方向ベクトル。各成分は `-1, 0, 1` のいずれか。
- `GameState`: 盤面 + 手番 + 手数。状態遷移の単位。
- `GameStatus`: 状態の評価ラベル（進行中/勝ち/引き分け）。
- 盤面キー: `board_key_base3` が返す `u128`。Empty=0, Black=1, White=2、走査順は z→y→x。
- `Outcome`: 探索中の局面を、次に手を打つプレイヤーから見た結果。現時点では手数を持たない。`GameStatus::InProgress` はまだ `Outcome` ではない。子局面の結果は `flip()` で親局面の視点に戻す。
- `MemoTable`: 盤面キーから `Outcome` を取り出すメモ化用の表。
- `solve`: 直前手による勝敗が未成立という前提で、局面を手番側から解く。
- `solve_after_move`: 最後に置いた `Position` を使って終局判定してから局面を解く。
- フロンティア: ある手数ちょうどで到達できる状態集合。`frontier_counts` では同一手数内だけ重複排除する。

## ルール（ゲーム）

- 4×4×4 の立方体盤面、重力あり。プレイヤーは柱 `(x, y)` を選ぶ。
- 縦・横・奥行・各種斜めのいずれかで 4 個並べば勝ち。判定は代表13方向。
- 先手は Black、交互着手。

## コード規約

- ドックコメント（`///`, `//!`）は日本語で書く。型と関数の役割、および「なぜそうしたか」を残す。
- 命名は英語、コメントは日本語。1 ファイル内で混在させない。
- `unwrap` / `expect` はライブラリ本体では使わない。テストと、論理的に到達不能な箇所のみ可。
- 失敗しうる処理は `Result` または `Option` を返す。
- `const fn` で書ける処理は `const fn` にする。
- 数値リテラル `4` を直書きしない。`BOARD_SIZE` などの定数を使う。
- 公開 API は `src/game/mod.rs` の `pub use` 経由でのみ出す。モジュール名のリーク禁止。
- 単体テストは `#[cfg(test)] mod tests` で対応モジュール内に置く。ライブラリ API 越しのテストは `tests/` に置く。
- `unsafe` は使わない。

## 作業の進め方

- 速度より読みやすさ優先。最適化は `benches/` で裏付けてから入れる。
- 設計判断（型の追加・表現の変更・外部依存追加・既存の冗長情報の削除）は ADR を 1 ファイル追加または更新してから実装する。
- 大きめの変更では AGENTS.md と該当 ADR を同じコミットで更新する。
- やらないこと: CI 設定の独断追加、外部依存の独断追加、`unsafe` の使用。

## ADR 索引

- ADR-0001 盤面表現を `[[[Cell; 4]; 4]; 4]` (`board[z][y][x]`) にする
- ADR-0002 `GameState` に `turn` と `moves_played` を冗長に持つ
- ADR-0003 盤面キーを 3進数 (`u128`, z→y→x 走査) で表す
- ADR-0004 勝敗判定は最後に置かれた `Position` 起点で代表13方向のみ調べる
- ADR-0005 探索結果のメモ化を `MemoTable` で始める
