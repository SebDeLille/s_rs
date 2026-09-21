# srsgtk

Ce crate fait partie du workspace [`s_rs`](../README.md) ; voir le README
racine pour une vue d'ensemble du projet.

Interface graphique GTK4 pour l'interpréteur `srs`, basée sur `libsrs`.
Elle inclut un canvas de dessin pilotable depuis Scheme et un REPL intégré
multi-ligne.

## Prérequis système

Ce crate dépend du binding Rust `gtk4`, qui nécessite les bibliothèques de
développement GTK4 installées sur le système (via `pkg-config`).

- Debian/Ubuntu : `sudo apt-get install libgtk-4-dev`
- Fedora : `sudo dnf install gtk4-devel`
- Arch Linux : `sudo pacman -S gtk4`

Une fois les paquets installés, la compilation se fait normalement :

```sh
cargo build -p srsgtk
cargo run -p srsgtk
```

## Documentation

- [`doc/graphics_api.md`](doc/graphics_api.md) : primitives Scheme
  spécifiques (non-R5RS) pour dessiner sur le canvas.

## Layout

- `gtk::ApplicationWindow` contenant un `gtk::Paned` en orientation
  verticale.
- Zone haute (~3/4 de la hauteur) : `gtk::DrawingArea`. Son rendu est
  piloté par la liste de commandes accumulées par les primitives Scheme de
  [`doc/graphics_api.md`](doc/graphics_api.md).
- Zone basse (~1/4 de la hauteur) : widget REPL implémenté dans
  `src/repl.rs` (zone de log, champ de saisie, continuation multi-ligne,
  historique avec flèches haut/bas).

## Démarrage

Au lancement, `srsgtk` :

1. charge le prélude Scheme pur `libs/function-samples.scm` ;
2. charge les scripts `*.scm` de `$HOME/.config/srs/startup/` puis de
   `.srs/startup/` (dans cet ordre) ;
3. affiche la fenêtre principale avec le canvas et le REPL.
