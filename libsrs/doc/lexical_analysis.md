```mermaid
stateDiagram-v2
    direction LR

    [*] --> Start

    classDef accepting fill:#fff,stroke:#333,stroke-width:2px
    classDef star fill:#fff,stroke:#f90,stroke-width:2px
    classDef error fill:#fff,stroke:#f00,stroke-width:2px

    class Start accepting
    class IntegerDone,FloatDone,IdDone,LtDone,GtDone star

    Start --> Start : whitespace
    Start --> Integer : 0-9
    Start --> Id : A-Z a-z _
    Start --> InString : \"
    Start --> Sharp : #
    Start --> [*] : ( ) + - * / ' =
    Start --> Lt : <
    Start --> Gt : >

    Integer --> Integer : 0-9
    Integer --> Start : WS / emit INTEGER
    Integer --> Float : .
    Integer --> Id : A-Z a-z _
    Integer --> IntegerDone : else / emit INTEGER, reprocess char

    Float --> Float : 0-9
    Float --> Start : WS / emit FLOAT
    Float --> FloatDone : else / emit FLOAT, reprocess char

    Id --> Id : A-Z a-z 0-9 _
    Id --> Start : WS / emit ID
    Id --> IdDone : else / emit ID, reprocess char

    InString --> InString : other
    InString --> Start : \" / emit STRING
    InString --> Escape : \

    Sharp --> Start : tT / emit TRUE
    Sharp --> Start : fF / emit FALSE
    Sharp --> Start : ( / emit SHARP

    Lt --> Start : = / emit LTE
    Lt --> LtDone : else / emit LT, reprocess char

    Gt --> Start : = / emit GTE
    Gt --> GtDone : else / emit GT, reprocess char

    Escape --> InString : \\ or \" / append
```

## Notes

- States ending in `Done` are transient markers used by the double-dispatch mechanism (`is_star`) in `filter`. They signal that the current character terminated the previous lexeme and must be processed again as the start of the next lexeme.
- All lexer states are defined as a single `LexerState` enum in `libsrs/src/interpretor/lexical_analyzer.rs`, replacing the legacy `u8` magic numbers.
