# libertyd

**La couche IA système de Liberty** — le cœur de son identité.

`libertyd` est un service système de première classe (et non une application)
dont **l'esprit est Claude** (compte Anthropic lié à la session) et les
réflexes des modèles légers/locaux. Voir [`../../docs/AI.md`](../../docs/AI.md).

Il :

- **initie** des actions au nom de l'utilisateur (l'inversion : tu ne promptes
  pas, tu réponds) avec le jugement calibré de Claude ;
- expose le **bus d'intents** de Liberty à Claude comme outils API (tool use) ;
- applique un **contrôle de capacités** strict sur toute action — y compris
  celles décidées par l'IA ;
- gère une **mémoire/contexte** utilisateur, stockée localement et chiffrée.

**Confidentialité :** pré-filtrage local, appels réseau journalisés et
révocables, mode dégradé hors-ligne. Voir la reformulation honnête dans
[`docs/AI.md`](../../docs/AI.md).

Statut : 🌱 prototype (Phase 4 de la roadmap).

## Architecture du code

| Module | Rôle |
|---|---|
| `effects.rs` | Effets déclarés + capacités (le garde-fou de l'OS) |
| `autonomy.rs` | Niveaux d'autonomie par domaine (le « curseur ») |
| `decision.rs` | `decide()` — le filet de sécurité sous l'IA, avec tests |
| `brain.rs` | Trait `Brain` : `ClaudeBrain` (auth compte Anthropic/clé API, requêtes `/v1/messages` exactes, testées) et `SimulatedBrain` (dev hors-ligne) |
| `main.rs` | Assemblage + rapport de démonstration |

## Lancer

```sh
cargo run  --manifest-path services/libertyd/Cargo.toml   # démo
cargo test --manifest-path services/libertyd/Cargo.toml   # 13 tests
```

Avec un compte Anthropic lié (`ANTHROPIC_AUTH_TOKEN`, OAuth) ou une clé API
(`ANTHROPIC_API_KEY`), `libertyd` le détecte et prépare les requêtes Claude
réelles (transport HTTP à câbler en phase réseau). Sans identifiants, la
simulation hors-ligne prend le relais.

La démo montre six situations initiées par l'IA sous deux profils d'autonomie,
avec la sécurité invariante : la télémétrie reste bloquée faute de capacité
réseau, même à 99 % de confiance.
