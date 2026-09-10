# libsrs

Bibliothèque implémentant l'interpréteur Scheme (sous-ensemble R5RS) utilisé
par les crates `srs` (REPL CLI) et `srsgtk` (interface graphique).

## Organisation

- `interpretor::lexical_analyzer` : analyse lexicale (tokenisation du code
  source).
- `interpretor::reader` : construction des expressions Scheme (S-expressions)
  à partir des tokens.
- `interpretor::evaluator` : évaluation des expressions (formes spéciales,
  procédures natives).
- `interpretor::repl` : pipeline d'évaluation partagé entre le REPL CLI et
  d'éventuels autres frontends (gère notamment les entrées incomplètes,
  cf. `EvalOutcome`).
- `interpretor::startup` : chargement des scripts de démarrage utilisateur
  (`*.scm` dans `$HOME/.config/srs/` et `.srs/` du répertoire courant).
- `types::core` : types Scheme (`SrsValue`, `Env`, ...).

## Tests

```sh
cargo test -p libsrs
```

Les tests incluent :
- des suites basées sur le rapport R5RS (`tests/r5rs/`) : nombres,
  procédures, paires, vecteurs, entrées/sorties ;
- des tests sur des fixtures Scheme (`tests/fixtures/scheme/`, ex.
  `factoriel.scm`) ;
- des tests unitaires sur le lexer, le reader, l'évaluateur et le
  chargement des scripts de démarrage.

## Documentation

- [`doc/lexical_analysis.md`](doc/lexical_analysis.md) : détails sur
  l'analyse lexicale.
- [`doc/r5rs_subset.md`](doc/r5rs_subset.md) : sous-ensemble de R5RS
  supporté (formes spéciales, primitives, types, écarts vs le standard).
