# ADR-0002: `GameState` に `turn` と `moves_played` を冗長に持つ

- Status: Accepted
- Date: 2026-05-04

## 背景

現ルール（先手は黒、交互着手、コマは消えない）の下では、`turn` と `moves_played` は `board` から復元できる。

- `moves_played` = 盤面上の非空マス数
- `turn` = `moves_played` の偶奇

そのため、状態として明示的に持つかは設計判断になる。

## 決定

`GameState` に `board` だけでなく `turn: Player` と `moves_played: u8` を持つ。

## 理由

- 状態遷移コードの可読性を取る。`play` 内で「次の手番に進める」「手数を 1 増やす」がそのままフィールド更新として書ける。
- `is_full` などの頻出判定がカウンタ参照だけで済む。
- 学習者が `GameState` のフィールド一覧を見たときに「ゲームの状態に必要な情報」が一目で揃う。

## 影響

- 状態数値化や完全解析の段階で冗長さが問題になる可能性がある。その時点で `board` を真実として `turn` と `moves_played` を計算で出す設計に切り替える。切り替え時には本 ADR を Superseded にし、後続 ADR で理由を残す。
- 外部から状態を復元する `from_board_key_base3` のような処理は、`board` から `turn` と `moves_played` を導出する責務を持つ。
- `GameState` を直接フィールド初期化するテストでは、`board` と `turn` / `moves_played` の整合を呼び出し側の責任で保つ必要がある。
