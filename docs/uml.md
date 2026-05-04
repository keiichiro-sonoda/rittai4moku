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
        +board_key_base3() u128
        +from_board_key_base3(u128) Option~GameState~
        +next_empty_z(Column) Option~usize~
        +is_column_full(Column) bool
        +legal_moves() Vec~Column~
        +cell_at(Position) Cell
        +count_same_cells(Position, Direction, Cell) usize
        +count_line_cells(Position, Direction, Cell) usize
        +is_winning_line(Position, Direction) bool
        +is_winning_position(Position) bool
        +status_after_move(Position) GameStatus
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
        +base3_digit() u128
        +from_base3_digit(u128) Option~Cell~
        +player() Option~Player~
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

    class GameStatus {
        <<enum>>
        InProgress
        Win(Player)
        Draw
    }

    class Outcome {
        <<enum>>
        Win
        Loss
        Draw
        +flip() Outcome
        +from_status_for_player(GameStatus, Player) Option~Outcome~
    }

    class MemoTable {
        +new() MemoTable
        +len() usize
        +is_empty() bool
        +remember(GameState, Outcome) Option~Outcome~
        +lookup(GameState) Option~Outcome~
        +contains(GameState) bool
    }

    GameState --> Board
    GameState --> Player
    GameState ..> Column
    GameState ..> Position
    GameState ..> Direction
    GameState ..> Cell
    GameState ..> PlayResult
    GameState ..> PlayError
    GameState ..> GameStatus
    Board --> Cell
    Player ..> Cell
    Cell ..> Player
    PlayResult --> GameState
    PlayResult --> Position
    Direction ..> Position
    MemoTable ..> GameState
    MemoTable --> Outcome
    Outcome ..> GameStatus
    Outcome ..> Player
```

## モジュールの関係

```mermaid
flowchart TD
    main["src/main.rs\n実行入口"]
    lib["src/lib.rs\nライブラリ入口"]
    game["src/game/mod.rs\ngame の公開窓口"]
    solver["src/solver/mod.rs\nsolver の公開窓口"]

    constants["constants.rs\n盤面サイズなど"]
    cell["cell.rs\nCell"]
    player["player.rs\nPlayer"]
    coordinate["coordinate.rs\nColumn / Position"]
    line["line.rs\nDirection"]
    status["status.rs\nGameStatus"]
    state["state.rs\nGameState / PlayResult / PlayError"]
    outcome["outcome.rs\nOutcome"]
    memo["memo.rs\nMemoTable"]

    main --> lib
    lib --> game
    lib --> solver
    game --> constants
    game --> cell
    game --> player
    game --> coordinate
    game --> line
    game --> status
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
    state --> status

    solver --> outcome
    solver --> memo
    memo --> game
    memo --> outcome
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
    target["起点の Cell を1回だけ確認"]
    empty{"Cell::Empty？"}
    dirs["ALL_DIRECTIONS\n代表13方向"]
    forward["direction 方向へ\ncount_same_cells"]
    backward["opposite 方向へ\ncount_same_cells"]
    total["forward + backward + 1"]
    win{"4個以上？"}

    placed --> target
    target --> empty
    empty -->|yes| resultNotWin["勝ちではない"]
    empty -->|no| dirs
    dirs --> forward
    dirs --> backward
    forward --> total
    backward --> total
    total --> win
    win -->|yes| resultWin["勝ち"]
    win -->|no| resultContinue["その方向では勝ちではない"]
```

この最後の図の流れは、`GameState::is_winning_position` として実装済みです。次はこの判定を使って、ゲーム全体が進行中・勝ち・引き分けのどれかを表す型へ進みます。

位置ごとに「この方向では4つ並びようがない」と分かる場合、その方向を事前に省く最適化も考えられます。ただし、今は勝敗判定の正しさと理解しやすさを優先し、その最適化は後回しにします。

## ゲーム状態の判定

```mermaid
flowchart TD
    placed["最後に置かれた Position"]
    win{"is_winning_position(placed) ?"}
    winner["cell_at(placed).player()"]
    full{"is_full() ?"}
    statusWin["GameStatus::Win(Player)"]
    statusDraw["GameStatus::Draw"]
    statusProgress["GameStatus::InProgress"]

    placed --> win
    win -->|yes| winner
    winner --> statusWin
    win -->|no| full
    full -->|yes| statusDraw
    full -->|no| statusProgress
```

この図の流れは、`GameState::status_after_move` として実装済みです。

勝敗判定のテストでは、通常の `play` で作る局面に加えて、盤面を直接組み立てるテストも使います。重力ありルールでは斜め方向の特定配置を合法手だけで作る準備が複雑になるため、勝敗判定ロジック単体を確認したい場合は直接盤面を作ります。

## 盤面キーの生成

```mermaid
flowchart TD
    start["board_key_base3"]
    order["z -> y -> x の順に走査\nxが最速で進む"]
    digit["Cell::base3_digit\nEmpty=0 Black=1 White=2"]
    place["place = 1, 3, 9, ..."]
    add["key += digit * place"]
    next["place *= 3"]
    done["u128 の盤面キー"]

    start --> order
    order --> digit
    digit --> place
    place --> add
    add --> next
    next --> order
    order --> done
```

最初のマス `(0, 0, 0)` は3進数の最下位桁として扱います。走査順は `(0,0,0)`, `(1,0,0)`, ..., `(3,3,0)`, `(0,0,1)`, ... です。

## 盤面キーからの復元

```mermaid
flowchart TD
    start["from_board_key_base3(key)"]
    digit["digit = key % 3"]
    cell["Cell::from_base3_digit(digit)"]
    invalidDigit{"0, 1, 2 以外？"}
    set["board[z][y][x] = cell"]
    count["空でなければ moves_played += 1"]
    shift["key /= 3"]
    extra{"64マスを読んだ後も key が残る？"}
    turn["moves_played の偶奇から turn を復元"]
    done["Some(GameState)"]
    none["None"]

    start --> digit
    digit --> cell
    cell --> invalidDigit
    invalidDigit -->|yes| none
    invalidDigit -->|no| set
    set --> count
    count --> shift
    shift --> extra
    extra -->|yes| none
    extra -->|no| turn
    turn --> done
```

復元処理は、保存したキーを再び `GameState` として扱うための準備です。ただし、現在の `from_board_key_base3` は重力に反していないか、黒白の個数が合法かまでは検証しません。初期状態から `play` で作った局面を保存し、そのキーを読み戻す用途を想定しています。

## メモ化の最小単位

```mermaid
flowchart TD
    state["GameState"]
    key["board_key_base3()"]
    table["MemoTable\nHashMap<u128, Outcome>"]
    hit{"保存済み？"}
    lookup["lookup(state)\nSome(Outcome)"]
    miss["lookup(state)\nNone"]
    calc["将来: この局面を探索する"]
    remember["remember(state, outcome)"]

    state --> key
    key --> table
    table --> hit
    hit -->|yes| lookup
    hit -->|no| miss
    miss --> calc
    calc --> remember
    remember --> table
```

メモ化は「同じ局面をもう一度調べない」ための仕組みです。現在は探索本体をまだ実装せず、`MemoTable` で盤面キーと `Outcome` を保存・取得するところだけを確認します。

## GameStatus から Outcome への変換

```mermaid
flowchart TD
    status["GameStatus"]
    perspective["評価する Player"]
    inProgress{"InProgress？"}
    draw{"Draw？"}
    win["Win(winner)"]
    same{"winner == perspective？"}
    none["None"]
    outcomeDraw["Some(Outcome::Draw)"]
    outcomeWin["Some(Outcome::Win)"]
    outcomeLoss["Some(Outcome::Loss)"]

    status --> inProgress
    perspective --> same
    inProgress -->|yes| none
    inProgress -->|no| draw
    draw -->|yes| outcomeDraw
    draw -->|no| win
    win --> same
    same -->|yes| outcomeWin
    same -->|no| outcomeLoss
```

`GameStatus` は「ゲームとしてどうなっているか」を表し、`Outcome` は「指定したプレイヤーから見てどうか」を表します。そのため、勝ち状態を `Outcome` に変換するときは、勝者と評価するプレイヤーを比較します。進行中の状態はまだ探索結果ではないので `None` にします。

## Outcome の視点反転

```mermaid
flowchart LR
    child["子局面の Outcome\n相手視点"]
    flip["flip()"]
    parent["親局面の Outcome\n自分視点"]

    win["Win"]
    loss["Loss"]
    draw["Draw"]
    flippedLoss["Loss"]
    flippedWin["Win"]
    flippedDraw["Draw"]

    child --> flip --> parent
    win --> flippedLoss
    loss --> flippedWin
    draw --> flippedDraw
```

自分が1手打った後の局面は相手番です。その子局面を解いて返ってくる `Outcome` は相手から見た結果なので、親局面で読むときは `flip()` で視点を戻します。
