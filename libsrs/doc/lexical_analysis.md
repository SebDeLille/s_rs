```mermaid
stateDiagram-v2
    direction LR

    [*] --> Start

    classDef accepting fill:#fff,stroke:#333,stroke-width:2px
    classDef star fill:#fff,stroke:#f90,stroke-width:2px
    classDef error fill:#fff,stroke:#f00,stroke-width:2px

    class Start accepting
    class IntegerDone,FloatDone,IdDone,ExponentDone,DotDone,SignDone,CharDone star

    %% --- whitespace & comments ---
    Start --> Start : whitespace
    Start --> LineComment : ;
    LineComment --> LineComment : any except newline
    LineComment --> Start : newline

    Start --> Sharp : #
    Sharp --> BlockComment : |
    BlockComment --> BlockComment : any (depth-counted internally)
    BlockComment --> BlockCommentBar : |
    BlockCommentBar --> Start : # / close (depth--, if 0 emit nothing)
    BlockCommentBar --> BlockComment : else
    BlockComment --> BlockCommentHash : #
    BlockCommentHash --> BlockComment : | / nested open (depth++)
    BlockCommentHash --> BlockComment : else

    %% --- punctuation ---
    Start --> [*] : ( / emit LPAREN
    Start --> [*] : ) / emit RPAREN
    Start --> [*] : ' / emit QUOTE
    Start --> [*] : ` / emit QUASIQUOTE
    Start --> Unquote : ,
    Unquote --> [*] : @ / emit UNQUOTE_SPLICING
    Unquote --> Start : WS or delimiter / emit UNQUOTE
    Unquote --> UnquoteDone : else / emit UNQUOTE, reprocess char

    %% --- sharp dispatch ---
    Sharp --> Start : tT / emit TRUE
    Sharp --> Start : fF / emit FALSE
    Sharp --> Start : ( / emit VECTOR_OPEN
    Sharp --> CharSharp : \

    CharSharp --> CharName : A-Za-z
    CharSharp --> Start : any other single char / emit CHAR
    CharName --> CharName : A-Za-z
    CharName --> Start : WS or delimiter / emit CHAR (resolve "space"/"newline", else error)
    CharName --> CharDone : else / emit CHAR, reprocess char

    %% --- general identifiers ---
    Start --> Id : letter | ! $ % & * / : < = > ? ^ _ ~
    Id --> Id : letter | digit | ! $ % & * / : < = > ? ^ _ ~ | + - . @
    Id --> Start : WS or delimiter / emit ID
    Id --> IdDone : else / emit ID, reprocess char

    %% --- sign (+/-): peculiar identifier, signed number, or extended identifier ---
    Start --> Sign : + -
    Sign --> Start : WS or delimiter / emit ID
    Sign --> Integer : 0-9
    Sign --> DotStart : .
    Sign --> Id : letter | special-initial | special-subsequent (e.g. "->foo")
    Sign --> SignDone : delimiter char / emit ID, reprocess char

    %% --- isolated dot: DOT, "...", or float ".5" ---
    Start --> DotStart : .
    DotStart --> Float : 0-9
    DotStart --> DotDot : .
    DotDot --> Start : . / emit ID "..."
    DotStart --> Start : WS or delimiter / emit DOT
    DotStart --> DotDone : else / emit DOT, reprocess char

    %% --- numbers ---
    Integer --> Integer : 0-9
    Integer --> Float : .
    Integer --> Exponent : eE
    Integer --> Id : letter | special-initial | special-subsequent (e.g. "1+")
    Integer --> Start : WS or delimiter / emit INTEGER
    Integer --> IntegerDone : else / emit INTEGER, reprocess char

    Float --> Float : 0-9
    Float --> Exponent : eE
    Float --> Start : WS or delimiter / emit FLOAT
    Float --> FloatDone : else / emit FLOAT, reprocess char

    Exponent --> ExponentDigits : 0-9
    Exponent --> ExponentSign : + -
    ExponentSign --> ExponentDigits : 0-9
    ExponentDigits --> ExponentDigits : 0-9
    ExponentDigits --> Start : WS or delimiter / emit FLOAT
    ExponentDigits --> ExponentDone : else / emit FLOAT, reprocess char

    %% --- strings ---
    Start --> InString : \"
    InString --> InString : other
    InString --> Start : \" / emit STRING
    InString --> Escape : \\
    Escape --> InString : \\ \" n t r / append resolved escape
```

## Notes

- States ending in `Done` are transient markers used by the double-dispatch mechanism (`is_star`) in `filter`. They signal that the current character terminated the previous lexeme and must be processed again as the start of the next lexeme.
- All lexer states are defined as a single `LexerState` enum in `libsrs/src/interpretor/lexical_analyzer.rs`, replacing the legacy `u8` magic numbers.
- `BlockComment` requires an internal depth counter to correctly match nested `#| ... #| ... |# ... |#` sequences; it cannot be modeled as a pure finite automaton with one state per nesting level.
- Identifiers starting with `+` or `-` are handled pragmatically (beyond strict R5RS grammar) to accept common real-world identifiers such as `->string`, `1+`, `list->vector`, matching the behavior of implementations like MIT Scheme and Guile.
- Number support in this subset covers signed integers/floats and exponent notation (`1e10`, `-3.2e-5`). Rationals (`1/2`) and radix/exactness prefixes (`#x`, `#e`, `#i`, `#b`, `#o`, `#d`) are explicitly out of scope for now.
- Datum comments (`#;`) are intentionally excluded from this diagram: they require discarding the *next full datum*, not just the next token, so they cannot be resolved at the lexical level alone. This would need cooperation from the parser/translator if introduced later.
- Character literals support the minimal R5RS name set: `space` and `newline`, plus any single character after `#\`. An unrecognized multi-letter name (e.g. `#\foobar`) is an explicit lexical error, not a silent fallback.
</content>
