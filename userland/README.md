# userland/

L'espace utilisateur de Liberty — tout ce que l'on voit et touche.

- `compositor/` — compositeur Wayland en Rust (rendu, fenêtres, GPU).
- `shell/` — le shell Liberty : barre universelle, lanceur, gestion de
  fenêtres. L'entrée en **langage naturel** y est un citoyen de première
  classe.
- `ai-runtime/` — bibliothèques + SDK pour que les apps exposent des actions
  et consomment l'inférence locale de `libertyd`.

Voir [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md).
