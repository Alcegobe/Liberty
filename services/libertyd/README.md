# libertyd

**La couche IA système de Liberty** — le cœur de son identité.

`libertyd` est un service système de première classe (et non une application)
dont **l'esprit est Claude** (Fable 5 par défaut, et chaque modèle plus
capable dès qu'il sort). Voir [`../../docs/AI.md`](../../docs/AI.md).

Il :

- **initie** des actions au nom de l'utilisateur (l'inversion : tu ne promptes
  pas, tu réponds) avec le jugement calibré de Claude ;
- **observe** le système par ses capteurs locaux (charge, mémoire, disques,
  services, journaux) et n'envoie que des rapports minimisés ;
- **agit** par une boucle agentique multi-tours (`observe` · `act` ·
  `ask_user` · `done`) — chaque action passée par `decide()` ;
- applique un **contrôle de capacités** strict sur toute action — y compris
  celles décidées par l'IA — et **journalise tout** (avec undo).

**Confidentialité :** pré-filtrage local, appels réseau journalisés et
révocables, mode dégradé hors-ligne. Voir la reformulation honnête dans
[`docs/AI.md`](../../docs/AI.md).

Statut : ⚙ v0.2 — l'esprit vivant (Phase 1 de la roadmap, faite).

## Les deux binaires

| Binaire | Rôle |
|---|---|
| `libertyd` | Le démon : `--daemon` (battement de cœur autonome), `--once` (un battement), `--demo` (boucle de décision hors-ligne) |
| `lish` | Le Liberty Shell : intentions en langage naturel, `!` pour le shell brut, `:journal`, `:undo`, `:caps` |

## Architecture du code

| Module | Rôle |
|---|---|
| `effects.rs` | Effets déclarés + capacités (le garde-fou de l'OS) |
| `autonomy.rs` | Niveaux d'autonomie par domaine (le « curseur ») |
| `decision.rs` | `decide()` — le filet de sécurité sous l'IA, avec tests |
| `config.rs` | `/etc/liberty/liberty.toml` : profil, capacités, modèle, rythme |
| `sensors.rs` | Les sens : sondes locales en lecture seule → rapport de situation |
| `executor.rs` | Les mains : observation (liste blanche) vs action journalisée |
| `journal.rs` | La mémoire auditable : JSONL append-only, undo |
| `brain.rs` | Identité de l'esprit + identifiants (clé API / OAuth) |
| `model.rs` | Politique de modèle : Fable 5 en tête, découverte à l'exécution |
| `agent.rs` | La boucle agentique multi-tours (feature `claude`) |
| `transport.rs` | HTTP vers l'API Anthropic, reprise sur erreurs (feature `claude`) |
| `main.rs` / `bin/lish.rs` | Le démon et le shell |

## Lancer

```sh
cd services/libertyd
cargo run -- --demo                       # démo hors-ligne, sans réseau
cargo test                                # la suite de tests
export ANTHROPIC_API_KEY=sk-ant-...
cargo run --features claude -- --once     # un vrai battement de cœur
cargo run --features claude -- --daemon   # la boucle autonome
cargo run --features claude --bin lish    # le shell en langage naturel
```

Sans identifiants ni `--features claude`, la simulation hors-ligne prend le
relais : la démo montre six situations initiées par l'IA sous deux profils
d'autonomie, avec la sécurité invariante — la télémétrie reste bloquée faute
de capacité réseau, même à 99 % de confiance.
