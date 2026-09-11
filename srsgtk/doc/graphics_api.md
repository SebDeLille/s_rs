# API graphique srsgtk (non-R5RS)

`srsgtk` ajoute à l'environnement global de `libsrs` des primitives Scheme
spécifiques pour dessiner sur le `gtk4::DrawingArea` (via Cairo). Elles ne
font pas partie de R5RS et ne sont disponibles que dans `srsgtk`.

Installées par `graphics.rs::install()`, sur un modèle « replay de
commandes » : chaque appel enregistre un `DrawCommand`, rejoué au moment du
rendu.

## Primitives

| Primitive | Signature | Description |
|---|---|---|
| `clear-canvas` | `()` | Vide la liste des commandes de dessin enregistrées |
| `set-color` | `(r g b)` | Couleur courante pour les dessins suivants, composantes flottantes 0..1 |
| `draw-line` | `(x1 y1 x2 y2)` | Trace une ligne |
| `draw-point` | `(x y)` | Trace un point (disque plein de rayon fixe 2 px) |
| `draw-rect` | `(x y w h)` | Trace un rectangle |
| `draw-circle` | `(x y r)` | Trace un cercle |
| `canvas-width` | `()` | Largeur courante du canvas (Integer) |
| `canvas-height` | `()` | Hauteur courante du canvas (Integer) |
| `set-redraw-hook!` | `(proc)` | Enregistre `proc`, une lambda `(lambda (width height) ...)` appelée à chaque redimensionnement/redraw |

## Exemple

```scheme
(set-color 1.0 0.0 0.0)
(draw-rect 10 10 100 50)
(draw-circle 200 200 40)
(draw-point 50 50)
```

## Limitations connues

- Pas de primitives clavier/souris.
- Pas de primitives texte ou image.
- Le widget REPL GTK (`srsgtk/src/repl.rs`) est de l'UI hôte, pas une
  primitive Scheme : il utilise `eval_source` de `libsrs` comme n'importe
  quel frontend.

## Échantillonnage de fonctions (`function-samples`)

`function-samples` n'est **pas** une primitive Rust : c'est du Scheme pur
(`libs/function-samples.scm`), chargé une fois au démarrage par
`srsgtk/src/main.rs` (avant les scripts de `.srs/startup`), et utilisant
uniquement des procédures génériques déjà exposées par `libsrs` — dont les
extensions non-R5RS `nan?` et `finite?` (`libsrs/src/interpretor/evaluator/natives/arithmetic.rs`).

| Primitive | Signature | Description |
|---|---|---|
| `function-samples` | `(f xmin xmax n)` | Échantillonne `f` (procédure à un argument) en `n` points régulièrement espacés entre `xmin` et `xmax` (toujours convertis en `Float`, pour qu'une division par zéro dans `f` produise `+inf.0`/`+nan.0` plutôt qu'une erreur arithmétique exacte). Retourne une liste de segments, chaque segment étant une liste de paires `(x . y)` en ordre croissant de `x` |

Un point est ignoré, et le segment courant clos, dès que le résultat de `f`
n'est pas un nombre ou n'est pas fini (`nan?`/`finite?`) — ceci évite de
tracer un trait à travers une discontinuité ou une asymptote.

**Limitation connue** : l'interpréteur n'a aucun mécanisme d'exception
Scheme (pas de `guard`/`catch`/`call/cc`). Une erreur d'évaluation levée par
`f` elle-même (ex : mauvais type d'argument) n'est donc **pas** rattrapée et
se propage hors de `function-samples`, contrairement aux résultats non
finis qui eux sont filtrés.

### Exemple

```scheme
(define f (lambda (x) (/ 1.0 x)))
(function-samples f -2.0 2.0 5)
;; => (((-2.0 . -0.5) (-1.0 . -1.0))
;;     ((1.0 . 1.0) (2.0 . 0.5)))
;; (le point x = 0.0, où f produit +inf.0, est filtré et coupe le tracé
;; en deux segments)
```

