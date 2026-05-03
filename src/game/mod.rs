//! 立体4目並べのルールと状態表現をまとめるモジュール。
//!
//! `game` の下には、盤面・プレイヤー・座標・状態遷移など、
//! ゲームそのものを説明する型や関数を置く。

mod cell;
mod constants;
mod coordinate;
mod line;
mod player;
mod state;

pub use cell::Cell;
pub use constants::{BOARD_SIZE, CELL_COUNT, COLUMN_COUNT};
pub use coordinate::{Column, Position};
pub use line::{ALL_DIRECTIONS, Direction};
pub use player::Player;
pub use state::{Board, GameState, PlayError, PlayResult};

/// ゲーム開始時の状態を表す定数。
///
/// `GameState::initial()` は「初期状態を生成する関数」、
/// `INITIAL_STATE` は「初期状態そのものを表す名前」として使える。
/// どちらの形も見比べながら、Rust の定数と関数の使い分けを学ぶために残している。
pub const INITIAL_STATE: GameState = GameState::initial();
