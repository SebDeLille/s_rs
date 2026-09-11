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
