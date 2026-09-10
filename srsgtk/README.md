# srsgtk

Ce crate fait partie du workspace [`s_rs`](../README.md) ; voir le README
racine pour une vue d'ensemble du projet.

Interface graphique GTK4 pour l'interpréteur `srs`, basée sur `libsrs`
(squelette : fenêtre + layout, sans logique Scheme branchée pour le moment).

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
- Zone haute (~3/4 de la hauteur) : `gtk::DrawingArea`, actuellement un
  simple fond uni (placeholder pour les futures primitives de dessin).
- Zone basse (~1/4 de la hauteur) : label placeholder (le widget REPL réel
  fait l'objet d'une sous-issue dédiée).
