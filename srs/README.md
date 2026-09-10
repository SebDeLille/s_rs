# srs

REPL en ligne de commande pour l'interpréteur Scheme `libsrs`.

## Usage

```sh
cargo run -p srs
```

Le REPL lit une ligne à la fois (`srs> `) et affiche le résultat de chaque
expression évaluée. `Ctrl+D` (fin de flux) quitte le programme.

## Limitation connue

Le REPL n'accepte pas encore la saisie multi-ligne : une expression
incomplète sur une ligne (ex. une parenthèse non fermée) est rapportée comme
une erreur plutôt que de continuer la lecture sur la ligne suivante.
