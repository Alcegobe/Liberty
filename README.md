# Liberty

**Un système d'exploitation AI-first : Claude (Fable 5) est l'esprit natif du
système — il observe, décide, agit et rend compte.**

Liberty n'essaie pas de refaire Windows ou macOS. Il place l'intelligence
artificielle **au cœur** du système — pas comme une application ajoutée, mais
comme un service système de première classe (`libertyd`) qui pilote la machine
en continu, et un shell (`lish`) où l'on parle au système en langage naturel.

## Vision en une phrase

> Un OS où l'utilisateur décrit ce qu'il veut — et où le système, le reste du
> temps, se gère **tout seul** : il surveille, répare, range, optimise, et ne
> dérange l'humain que quand la décision lui appartient.

## Ce qui tourne aujourd'hui (v0.2)

| Composant | Rôle | État |
|---|---|---|
| **`libertyd --daemon`** | Le battement de cœur autonome : capteurs locaux → rapport de situation → Fable 5 → décision → action → journal | ✅ fonctionnel |
| **`lish`** | Le Liberty Shell : intentions en langage naturel, exécution agentique sous capacités, `!` pour le shell brut | ✅ fonctionnel |
| **Boucle de décision** | Capacités, niveaux d'autonomie, règle réversible/local, jugement calibré — le filet de sécurité *sous* l'IA | ✅ testé |
| **Exécuteur** | Observation en lecture seule (liste blanche) vs actions journalisées et annulables | ✅ testé |
| **Installation VM** | Une Debian minimale → Liberty en une commande | ✅ [`docs/INSTALL.md`](docs/INSTALL.md) |

```
┌────────────────────────────────────────────────────────┐
│  lish (langage naturel)        libertyd --daemon       │
│        │                              │                │
│        └────────── l'esprit ──────────┘                │
│              Claude · Fable 5 (et chaque modèle        │
│              plus capable, découvert à l'exécution)    │
│                        │                               │
│              outils : observe · act · ask_user · done  │
│                        │                               │
│      boucle de décision decide() — l'OS, pas l'IA :    │
│      capacités · autonomie · réversibilité · confiance │
│                        │                               │
│      exécuteur journalisé (undo) · capteurs locaux     │
│                        │                               │
│              Linux minimal (Debian aujourd'hui,        │
│              image Liberty dédiée demain)              │
└────────────────────────────────────────────────────────┘
```

## Philosophie

- **AI-first, AI-partout** : l'esprit voit le système entier (santé, disque,
  services, journaux) et agit en continu. L'interface humaine *est* le
  langage naturel.
- **L'inversion** : l'IA n'attend pas d'être promptée. Elle initie, propose,
  questionne — voir [`docs/INVERSION.md`](docs/INVERSION.md).
- **L'OS décide, pas le modèle** : chaque action passe par la boucle
  `decide()` (capacités accordées, profil d'autonomie, réversibilité,
  confiance calibrée). Même Fable 5 ne peut pas déborder de ce qui est
  accordé.
- **Tout est journalisé, beaucoup est annulable** : l'autonomie se paie en
  transparence (`lish` → `:journal`, `:undo`).
- **Toujours le meilleur Claude** : à chaque démarrage, Liberty interroge
  l'API Modèles et retient le plus capable accessible au compte. Un nouveau
  modèle sort → Liberty l'utilise, sans mise à jour.

## Démarrage rapide

**Dans une VM (recommandé)** — voir le guide complet
[`docs/INSTALL.md`](docs/INSTALL.md) :

```sh
# dans une Debian 12/13 minimale, en root :
apt-get update && apt-get install -y curl ca-certificates
curl -fsSL https://raw.githubusercontent.com/Alcegobe/Liberty/main/install/liberty-install.sh | sh
```

**En développement (n'importe quelle machine avec Rust)** :

```sh
cd services/libertyd
cargo run -- --demo                 # la boucle de décision, hors-ligne
cargo test                          # la suite de tests
export ANTHROPIC_API_KEY=sk-ant-...
cargo run --features claude -- --once    # un vrai battement de cœur
cargo run --features claude --bin lish   # le shell en langage naturel
```

## Documentation

- [`docs/VISION.md`](docs/VISION.md) — pourquoi Liberty existe
- [`docs/INVERSION.md`](docs/INVERSION.md) — l'IA initie, l'humain arbitre
- [`docs/AUTONOMY.md`](docs/AUTONOMY.md) — le curseur d'autonomie par domaine
- [`docs/AI.md`](docs/AI.md) — Claude comme esprit : modèles, auth, agent
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — les couches du système
- [`docs/INSTALL.md`](docs/INSTALL.md) — installation dans une machine virtuelle
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — le plan, phase par phase
