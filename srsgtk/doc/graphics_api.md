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

## Tracé de courbes (`libs/plot.scm`)

`libs/plot.scm` fournit une bibliothèque Scheme de tracé 2D s'appuyant sur
`function-samples` et sur les primitives canvas (`clear-canvas`, `set-color`,
`draw-line`, `draw-point`, `canvas-width`, `canvas-height`, `set-redraw-hook!`).
Elle se charge à la demande via `(load "plot")` une fois installée dans
`~/.config/srs/libs/` :

```sh
mkdir -p ~/.config/srs/libs
ln -sf "$PWD/libs/plot.scm" ~/.config/srs/libs/plot.scm
```

| Procédure | Signature | Description |
|---|---|---|
| `plot-function` | `(f xmin xmax)` | Trace `f` sur `[xmin, xmax]` avec repère auto-mis à l'échelle, coupure aux discontinuités/asymptotes et redessin automatique au redimensionnement |
| `plot-axes` | `(xmin xmax ymin ymax)` | Dessine le cadre du repère (axes sur `0.0` si l'intervalle le contient, sinon sur le bord le plus proche, avec graduations aux extrémités) |
| `plot-bounds` | `(segments)` | Calcule `(xmin xmax ymin ymax)` sur les segments fournis (`(-1.0 1.0 -1.0 1.0)` si vide) |
| `plot-segment` | `(segment xmin xmax ymin ymax margin)` | Trace un segment continu écrêté sur `[ymin, ymax]` (`draw-point` pour un point isolé, `draw-line` sinon) |
| `plot-scale-x` | `(x xmin xmax margin canvas-w)` | Convertit une abscisse de l'espace données vers les pixels du canvas |
| `plot-scale-y` | `(y ymin ymax margin canvas-h)` | Convertit une ordonnée vers les pixels du canvas (axe Y inversé pour Cairo) |

### Validation manuelle bout-en-bout

Les scripts de test se trouvent dans `srsgtk/tests/fixtures/`. Lancer
l'interface depuis la racine du dépôt :

```sh
cargo run -p srsgtk
```

Puis évaluer successivement chacun des scénarios dans l'entrée REPL en bas de
la fenêtre :

1. **Courbe continue (`cos`)** :
   ```scheme
   (load "srsgtk/tests/fixtures/plot_cos.scm")
   ```
   - Repère gris centré sur `(0, 0)` avec graduations aux extrémités.
   - Courbe continue de `cos(x)` sur `[-6.28, 6.28]` atteignant les bornes haute et basse du cadre.
   - Redimensionner la fenêtre (et déplacer le séparateur `Paned`) : le repère et la courbe se recalculent proportionnellement sans superposition d'anciens tracés.

2. **Asymptote verticale en `x = 0` (`1/x`)** :
   ```scheme
   (load "srsgtk/tests/fixtures/plot_inv.scm")
   ```
   - Deux branches hyperboliques distinctes sur `[-5, 5]` de part et d'autre de l'axe vertical `x = 0`.
   - Aucun trait vertical parasite reliant la branche `x < 0` (`y -> -inf`) à la branche `x > 0` (`y -> +inf`).
   - Échelle verticale lisible (les valeurs extrêmes proches du pôle sont écrêtées proprement aux marges du cadre).
   - Redimensionner la fenêtre : le tracé se redessine correctement aux nouvelles dimensions.

3. **Asymptotes périodiques hors grille (`tan`)** :
   ```scheme
   (load "srsgtk/tests/fixtures/plot_tan.scm")
   ```
   - Cinq branches distinctes de `tan(x)` sur `[-6.28, 6.28]`, séparées aux quatre asymptotes `±π/2` et `±3π/2` bien que la grille d'échantillonnage ne tombe pas exactement sur ces abscisses irrationnelles.
   - Aucun segment parasite traversant les asymptotes ; écrêtage net en haut et en bas du cadre.
   - Redimensionner la fenêtre : le tracé se met à jour sans artefact.
