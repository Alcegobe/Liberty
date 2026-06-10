# Liberty

**Un système d'exploitation AI-first, moderne, épuré et ultra-optimisé.**

Liberty n'essaie pas de refaire Windows ou macOS. Il s'inspire de ce qu'ils
ont de meilleur, jette ce qui est vieux, et place l'**intelligence
artificielle au cœur du système** — pas comme une application ajoutée, mais
comme une couche native qui pilote l'interface, les fichiers et les
automatisations.

## Vision en une phrase

> Un OS où l'utilisateur décrit ce qu'il veut, et où le système le fait —
> rapide, sobre, prévisible, et respectueux de la vie privée (IA locale par
> défaut).

## Philosophie

- **AI-first** : l'IA est un service système de première classe (`libertyd`),
  pas un gadget. Local par défaut, cloud optionnel et explicite.
- **Épuré** : une seule bonne façon de faire chaque chose. Pas de couches
  d'héritage de 30 ans.
- **Ultra-optimisé** : Rust partout dans le userland, démarrage rapide,
  empreinte mémoire minimale, latence faible.
- **Sûr** : mémoire sûre (Rust), capacités plutôt que permissions globales,
  sandbox par défaut.

## Approche technique (pragmatique)

On ne réécrit **pas** un noyau de zéro. On part d'un **noyau Linux minimal**
(drivers, ordonnanceur, gestion mémoire matures et gratuits) et on construit
**au-dessus** tout ce qui fait l'identité de Liberty :

| Couche               | Choix                                   |
|----------------------|-----------------------------------------|
| Noyau                | Linux minimal (LTS), config sur-mesure  |
| Langage userland     | Rust                                    |
| Affichage            | Compositeur Wayland maison              |
| Shell / UI           | Shell Liberty (graphique + langage nat.)|
| IA système           | `libertyd` + **Claude** (compte Anthropic) |
| Paquets              | Gestionnaire déclaratif, immuable        |
| Cibles               | x86_64, ARM64 (test via QEMU)           |

## État du projet

🌱 **Phase 0 — Fondations.** Le projet démarre. Voir
[`docs/ROADMAP.md`](docs/ROADMAP.md) pour le plan détaillé et
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) pour la conception.

## Documentation

- [Vision](docs/VISION.md) — le « pourquoi », et les quatre promesses à l'humain.
- [L'inversion](docs/INVERSION.md) — le paradigme central : l'IA initie, l'humain répond.
- [L'IA de Liberty](docs/AI.md) — Claude comme esprit du système, connexion par compte Anthropic.
- [Autonomie](docs/AUTONOMY.md) — comment l'IA agit seule, en sécurité, et réglable.
- [Architecture](docs/ARCHITECTURE.md) — le « comment » technique.
- [Le langage Lib](docs/LANGUAGE.md) — le langage d'action pensé pour l'IA.
- [Roadmap](docs/ROADMAP.md) — les phases et jalons.

## Licence

À définir (proposition : licence permissive type MIT/Apache-2.0 pour
maximiser l'adoption).
