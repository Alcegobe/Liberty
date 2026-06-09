# libertyd

**La couche IA système de Liberty** — le cœur de son identité.

`libertyd` est un service système de première classe (et non une application)
qui :

- exécute un **runtime d'inférence local** (CPU/GPU/NPU), backend modulaire ;
- expose un **bus d'intents** : les applications déclarent des actions, et
  `libertyd` les découvre et les compose pour répondre à des requêtes en
  langage naturel ;
- applique un **contrôle de capacités** strict sur toute action ;
- gère une **mémoire/contexte** utilisateur, stockée localement et chiffrée.

**Confidentialité :** local par défaut. Tout appel réseau est explicite,
visible et révocable.

Statut : 🌱 conception (Phase 4 de la roadmap).

## Prototype exécutable — la boucle de décision

Un premier prototype, sans dépendance, montre déjà le cœur de la vision :
l'inversion (l'IA initie), les niveaux d'autonomie réglables, la règle
« réversible + local → silencieux », les capacités comme garde-fou, le
jugement calibré (consulter dans le doute) et le journal transparent.

```sh
cargo run --manifest-path services/libertyd/Cargo.toml
```

Il simule six situations détectées par l'IA, sous deux profils d'autonomie
(« Prudent » et « Confiance »), et montre comment la décision change — tout
en gardant la sécurité invariante (la télémétrie reste bloquée faute de
capacité réseau, même à 99 % de confiance).

Voir [`src/main.rs`](src/main.rs) : la fonction `decide()` concentre toute la
philosophie de Liberty.
