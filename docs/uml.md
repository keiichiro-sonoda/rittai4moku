# UML / Mermaid 図

## この文書の目的

この文書では、現在のコード構造を Mermaid 図で可視化します。

Rust でも UML のような設計図を Markdown に残せます。ここでは厳密な UML 記法にこだわりすぎず、学習用に「型同士の関係」「モジュールの依存」「処理の流れ」が分かることを優先します。

GitHub や Mermaid 対応エディタでは、以下のコードブロックが図として表示されます。

## 型の関係

```mermaid
classDiagram
    class GameState {
        +Board board
        +Player turn
        +u8 moves_played
        +initial() GameState
        +is_full() bool
        +next_empty_z(Column) Option~usize~
        +is_column_full(Column) bool
        +legal_moves() Vec~Column~
        +cell_at(Position) Cell
        +count_same_cells(Position, Direction, Cell) usize
        +play(Column) Result~PlayResult, PlayError~
    }

    class Board {
        <<type alias>>
        [[[Cell; 4]; 4]; 4]
    }

    class Cell {
        <<enum>>
        Empty
        White
        Black
    }

    class Player {
        <<enum>>
        White
        Black
        +cell() Cell
        +next() Player
    }

    class Column {
        +usize x
        +usize y
        +new(usize, usize) Column
        +is_in_bounds() bool
    }

    class Position {
        +usize x
        +usize y
        +usize z
        +new(usize, usize, usize) Position
    }

    class Direction {
        +isize dx
        +isize dy
        +isize dz
        +new(isize, isize, isize) Direction
        +opposite() Direction
        +step_from(Position) Option~Position~
    }

    class PlayResult {
        +GameState state
        +Position placed_at
    }

    class PlayError {
        <<enum>>
        OutOfBounds
        ColumnFull
    }

    GameState --> Board
    GameState --> Player
    GameState ..> Column
    GameState ..> Position
    GameState ..> Direction
    GameState ..> Cell
    GameState ..> PlayResult
    GameState ..> PlayError
    Board --> Cell
    Player ..> Cell
    PlayResult --> GameState
    PlayResult --> Position
    Direction ..> Position
```

## モジュールの関係

```mermaid
flowchart TD
    main["src/main.rs\n実行入口"]
    lib["src/lib.rs\nライブラリ入口"]
    game["src/game/mod.rs\ngame の公開窓口"]

    constants["constants.rs\n盤面サイズなど"]
    cell["cell.rs\nCell"]
    player["player.rs\nPlayer"]
    coordinate["coordinate.rs\nColumn / Position"]
    line["line.rs\nDirection"]
    state["state.rs\nGameState / PlayResult / PlayError"]

    main --> lib
    lib --> game
    game --> constants
    game --> cell
    game --> player
    game --> coordinate
    game --> line
    game --> state

    player --> cell
    coordinate --> constants
    line --> constants
    line --> coordinate
    state --> constants
    state --> cell
    state --> player
    state --> coordinate
    state --> line
```

## 着手処理の流れ

```mermaid
sequenceDiagram
    participant Caller as 呼び出し側
    participant State as GameState
    participant Column as Column
    participant Board as Board

    Caller->>State: play(column)
    State->>Column: is_in_bounds()
    alt 柱が盤面外
        State-->>Caller: Err(PlayError::OutOfBounds)
    else 柱が盤面内
        State->>State: next_empty_z(column)
        State->>Board: board[z][y][x] を下から確認
        alt 柱が満杯
            State-->>Caller: Err(PlayError::ColumnFull)
        else 空き高さ z が見つかる
            State->>Board: board[z][y][x] = turn.cell()
            State->>State: turn = turn.next()
            State->>State: moves_played += 1
            State-->>Caller: Ok(PlayResult { state, placed_at })
        end
    end
```

## 勝敗判定へ進むための流れ

```mermaid
flowchart TD
    placed["最後に置かれた Position"]
    dirs["ALL_DIRECTIONS\n代表13方向"]
    forward["direction 方向へ\ncount_same_cells"]
    backward["opposite 方向へ\ncount_same_cells"]
    total["forward + backward + 1"]
    win{"4個以上？"}

    placed --> dirs
    dirs --> forward
    dirs --> backward
    forward --> total
    backward --> total
    total --> win
    win -->|yes| resultWin["勝ち"]
    win -->|no| resultContinue["その方向では勝ちではない"]
```

この最後の図は、まだ完全には実装していない次の段階を表します。現在は `count_same_cells` まで実装済みなので、次は正方向と逆方向を合計して4個以上か判定します。
