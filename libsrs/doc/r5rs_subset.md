# Sous-ensemble R5RS supporté

`libsrs` implémente un sous-ensemble volontairement restreint de R5RS. Ce
document liste ce qui est disponible aujourd'hui et ce qui ne l'est pas
encore, pour servir de référence aux utilisateurs et contributeurs.

## Formes spéciales

Dispatch dans `interpretor/evaluator/special_forms.rs::eval_combination`.

| Forme | Support | Notes |
|---|---|---|
| `define` | ✅ | `(define name expr)` uniquement — pas de sucre `(define (f x) ...)` |
| `lambda` | ✅ | paramètres fixes + reste variadique (dotted) |
| `let` | ✅ | pas de `let` nommé (named let) |
| `let*` | ✅ | |
| `do` | ✅ | |
| `if` | ✅ | |
| `quote` | ✅ | |
| `quasiquote` | ✅ | `unquote`, `unquote-splicing`, imbrication |
| `load` | ⚠️ extension | chargement de fichier, non standard R5RS. Les chemins contenant un `/` ou se terminant par `.scm` sont utilisés tels quels ; un nom nu (ex: `(load "csv")`) est cherché dans `~/.config/srs/libs/<nom>.scm` |

### Non implémentées

`cond`, `case`, `and`, `or`, `set!`, `letrec`/`letrec*`, `let` nommé,
`define-syntax`/`syntax-rules`/`let-syntax`, `delay`/`force`,
`call-with-current-continuation`/`call/cc`.

`dynamic-wind` est implémenté comme une native, mais de façon limitée (voir
section [Contrôle](#contrôle)).

> `Env::set` existe déjà dans `types/core.rs` mais n'est câblé à aucune forme
> spéciale : `set!` n'est donc pas utilisable pour l'instant.
>
> Le type `Promise` existe dans `types/core.rs` mais n'est utilisé par aucune
> primitive : `delay`/`force` ne sont pas fonctionnels.

## Procédures natives

Enregistrement central : `interpretor/evaluator/natives.rs::global_env()`.

### Arithmétique (`natives/arithmetic.rs`)

`+` `-` `*` `/` `=` `<` `>` `<=` `>=` `max` `not`
`exact->inexact` `inexact->exact` `exact?` `inexact?` `nan?` `finite?`

`nan?` et `finite?` ne sont pas R5RS (empruntées à R7RS) : elles testent la
valeur `f64` sous-jacente d'un nombre (`Integer`/`Rational` ne sont jamais
NaN ni infinis, seul `Float` peut l'être).

Absents : `min`, `quotient`, `remainder`, `modulo`, `abs`, `zero?`,
`positive?`, `negative?`, `odd?`, `even?`, `expt`, `sqrt`, `number?`,
`integer?`, `gcd`, `lcm`, `floor`, `ceiling`, `round`, `truncate`,
`numerator`, `denominator`.

### Processus (`natives/process.rs`)

`process-installed?` `exit` `system` `system*`

Il s'agit d'extensions non-R5RS : `exit` est inspirée de R7RS, tandis que
`system`/`system*` permettent d'exécuter des commandes externes.

### Trigonométrie (`natives/trig.rs`)

`sin` `cos` `tan` `atan` (1 argument seulement)

Absents : `atan` à 2 arguments, `exp`, `log`.

### Paires et listes (`natives/pairs.rs`)

`cons` `car` `cdr` `apply` `map` `length` `null?` `eq?` `list` `reverse`
`list-ref` `pair?` ainsi que les accesseurs composés R5RS `caar` … `cddddr`
(28 noms).

`eq?` compare l'identité des objets mutables (`Rc::ptr_eq`), et les
valeurs atomiques par valeur (deux nombres égaux comptent comme
égaux pour `eq?`, conformément à R5RS).

Absents : `list?`, `set-car!`, `set-cdr!`, `append`, `list-tail`,
`memq`/`memv`/`member`, `assq`/`assv`/`assoc`, `for-each`,
`eqv?`/`equal?`.

### Vecteurs (`natives/vectors.rs`)

`vector` `make-vector` `vector?` `vector-length` `vector-ref` `vector-set!`
`vector->list` `list->vector` `vector-fill!`

Absents : `vector-map`, `vector-for-each`, `vector-copy`.

### Chaînes (`natives/strings.rs`)

`string?` `string-length` `string-ref` `string=?` `substring` `string-append`
`list->string` `char=?`

Absents : `string-set!`, `string->list`, `string-ci=?` et comparaisons
insensibles à la casse, `make-string`, `string-copy`, `string-fill!`.

### Contrôle (`natives/control.rs`)

`dynamic-wind` existe en version *downward-only* : `(dynamic-wind before thunk
after)` appelle `before`, puis `thunk`, puis `after`, et retourne la valeur de
`thunk`. Même si `thunk` lève une erreur, `after` est toujours exécuté avant de
re-propager l'erreur.

Limitation : sans `call/cc`, il n'y a pas de ré-entrance possible dans `thunk`
après l'exécution de `after` (pas de continuation capturable et invocable).

### Entrées/sorties (`natives/io.rs`)

`display` `newline` `write-char` `write-string` `read-char` `peek-char`
`read-line` `char-ready?` `eof-object` `eof-object?` `current-input-port`
`current-output-port` `open-input-string` `open-input-file` `open-output-string`
`get-output-string` `close-input-port`

Absents : `write`, `read`, `open-output-file`, etc.

### Totalement absent

- Strings : `string-set!`, `string->list`, `string-ci=?` et apparentes,
  `make-string`, `string-copy`, `string-fill!`
- Chars : `char?`, `char->integer`, `integer->char`, `char-alphabetic?`, ...
- Symboles : `symbol?`, `symbol->string`, `string->symbol`
- Booléens : `boolean?`
- Procédures : `procedure?`
- Égalité : `eqv?`, `equal?`

## Types Scheme (`types::core::SrsValue`)

- `Integer(i64)` — exact
- `Float(f64)` — inexact
- `Rational(i64, i64)` — exact rationnel, pas de bignum. La conversion
  `inexact->exact` réduit la fraction par le PGCD ; les littéraux
  rationnels ne sont pas supportés.
- `Boolean(bool)`
- `Character(char)`
- `String` — mutable (`Rc<RefCell<String>>`)
- `Symbol(String)`
- `Nil`
- `Pair` — mutable (`Rc<RefCell<(SrsValue, SrsValue)>>`), cycles possibles
- `Vector` — mutable (`Rc<RefCell<Vec<SrsValue>>>`)
- `Unspecified`, `Eof`
- `Procedure` (lambda), `Native`
- `Promise` — type défini mais mort (aucune primitive ne l'utilise)
- `Port` — port d'entrée/sortie (stdin/stdout, chaînes en mémoire)

Pas de nombres complexes.

## Autres écarts vs R5RS

- Pas de tail-call optimization : `eval`/`apply_lambda` sont des appels Rust
  récursifs classiques → risque de stack overflow sur boucles récursives
  profondes (pas de trampoline).
- Pas de continuations (`call/cc`). `dynamic-wind` est présent mais en version
  *downward-only* (pas de ré-entrance possible).
- Pas de macros hygiéniques.
- La forme `begin` est disponible et évalue ses expressions en séquence ;
  les corps de `lambda`, `let`, `let*` et `do` ont le même comportement
  (équivalent à un `begin` implicite).

## Périmètre couvert par les tests

`libsrs/tests/r5rs/` :

- `numerical_tests.rs` : `+`, `*`, `sin`/`cos`/`tan`/`atan`
- `pair_tests.rs` : `cons`, `car`, `cdr`, `length`, `null?`, `list`,
  `reverse`, `list-ref`, `pair?`, accesseurs composés `caar` … `cddddr`
- `procedure_tests.rs` : `lambda`, application, `let`, `let*`, `begin`
- `vector_tests.rs` : `vector`, `vector?`, `make-vector`, `vector-length`,
  `vector-ref`, `vector-set!`, `vector->list`, `list->vector`, `vector-fill!`
- `string_tests.rs` : `string?`, `string-length`, `string-ref`, `string=?`,
  `substring`, `string-append`, `list->string`, `char=?`
- `control_tests.rs` : `dynamic-wind` (ordre d'exécution, `after` en cas
  d'erreur du thunk, arité).
- `io_tests.rs` : `display`, `newline`, `write-char`, `write-string`, ports
  de chaînes (`open-input-string`, `open-output-string`, `get-output-string`),
  `open-input-file`, `close-input-port`, `read-char`, `peek-char`, `read-line`,
  `char-ready?`, `eof-object`, `current-input-port`, `current-output-port`
