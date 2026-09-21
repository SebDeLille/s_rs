# s_rs

Interpréteur Scheme minimal (sous-ensemble R5RS) en Rust, sans dépendance
runtime externe. Le projet est organisé en workspace Cargo.

## Structure du workspace

- [`libsrs`](libsrs/README.md) : le cœur de l'interpréteur (analyse
  lexicale, lecteur, évaluateur, types Scheme, scripts de démarrage).
- [`srs`](srs/README.md) : REPL en ligne de commande construit sur
  `libsrs`.
- [`srsgtk`](srsgtk/README.md) : interface graphique GTK4 branchée sur
  `libsrs` (canvas de dessin via Cairo et REPL intégré).

## Démarrage rapide

```sh
# Compiler tout le workspace
cargo build

# Lancer le REPL
cargo run -p srs

# Lancer les tests de libsrs (suites R5RS incluses)
cargo test -p libsrs
```

`srs` charge automatiquement les fichiers `*.scm` présents dans
`$HOME/.config/srs/startup/` puis dans `.srs/startup/` du répertoire courant
au démarrage. `srsgtk` charge également ces scripts après son prélude Scheme
(voir [`srsgtk/src/main.rs`](srsgtk/src/main.rs)). Pour les détails du
mécanisme, voir [`libsrs/src/interpretor/startup.rs`](libsrs/src/interpretor/startup.rs).

`srsgtk` nécessite les bibliothèques de développement GTK4 installées sur le
système ; voir [`srsgtk/README.md`](srsgtk/README.md) pour les prérequis par
distribution.

## Contribuer

Le workflow (branches, PR, `cargo fmt`/`cargo clippy`) est décrit dans
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Licence

Voir [`LICENSE`](LICENSE).
