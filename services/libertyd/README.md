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
