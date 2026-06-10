# L'IA de Liberty — Claude comme esprit du système

> Décision de pivot (juin 2026) : **l'intelligence de Liberty, c'est Claude.**
> On se connecte à l'OS avec son **compte Anthropic**, et `libertyd` parle à
> Claude via l'API. Ce document décrit l'architecture IA, l'authentification,
> et ce que ce pivot change — honnêtement — à la vision initiale.

## Pourquoi Claude (et pas un modèle local) comme cerveau

La version initiale de la vision disait « IA locale par défaut, cloud jamais
requis ». C'était cohérent, mais il faut être lucide sur le compromis :

- Le jugement calibré qui rend **l'inversion** sûre (savoir *quand* agir seul
  et *quand* consulter l'humain) exige un modèle de **niveau frontière**.
  Un petit modèle local se trompera sur exactement les cas qui comptent.
- Liberty est conçu *autour* de son IA. Si l'IA est médiocre, tout l'édifice
  (autonomie, langage Lib, bus d'intents) devient un beau cadre vide.

Donc : **Claude est l'esprit, le local est le réflexe.** C'est un changement
assumé de la vision, pas un glissement silencieux.

## Architecture à deux étages

| Étage | Modèle | Rôle |
|---|---|---|
| **L'esprit** | `claude-opus-4-8` (API Anthropic) | Jugement, décisions d'autonomie, langage naturel, composition d'intents, questions à l'humain |
| **Les réflexes** | `claude-haiku-4-5` (API) et/ou petits modèles locaux | Classification rapide d'événements, tri « trivial vs à remonter », embeddings, mode dégradé hors-ligne |

Règles de fonctionnement :

- **Pré-filtrage local.** La télémétrie brute (capteurs, fichiers, processus)
  est d'abord triée par les réflexes. Seules les *situations* — résumées,
  minimisées — montent vers Claude. On n'envoie jamais un flux brut.
- **Mode dégradé hors-ligne.** Sans réseau, Liberty reste un OS complet ;
  seuls les réflexes locaux tournent (actions 🟢 sûres et mécaniques). Les
  décisions de l'esprit attendent le retour du réseau.
- **Le garde-fou de capacités s'applique à Claude aussi.** Le contrôle
  d'effets/capacités est appliqué *par l'OS*, en dessous du modèle. Même
  l'esprit ne peut pas déborder de ce qui est accordé.

## La vie privée, reformulée honnêtement

L'ancienne promesse « ta vie ne part jamais dans le cloud » n'est plus exacte,
et on ne va pas faire semblant. La promesse devient :

1. **Ton compte, ton lien direct.** Les données vont chez Anthropic sous *ton*
   compte — pas chez un courtier publicitaire, pas revendues, pas croisées.
2. **Minimisation.** Le pré-filtrage local fait que Claude reçoit des
   situations résumées, pas tes disques entiers.
3. **Transparence totale.** Chaque appel réseau de `libertyd` est journalisé,
   consultable, et coupable d'un geste (mode hors-ligne).
4. **Le local reste possible.** Le backend est une interface (`Brain`) : un
   utilisateur souverain pourra brancher un modèle local à la place de Claude,
   en acceptant la perte de jugement.

## Connexion : « Se connecter avec son compte Anthropic »

L'ouverture de session Liberty inclut la liaison du compte Anthropic. Deux
mécanismes, dans l'ordre de préférence :

### 1. OAuth (recommandé — c'est le « login » grand public)
- Au premier démarrage, Liberty ouvre le flux OAuth d'Anthropic dans le
  navigateur (ou affiche une URL + code sur les machines sans navigateur,
  façon `ant auth login --no-browser`).
- L'OS reçoit un **jeton d'accès court** + un jeton de rafraîchissement,
  stockés dans le **trousseau chiffré** de Liberty (jamais en clair).
- Les requêtes API portent `Authorization: Bearer <token>` **plus l'en-tête
  `anthropic-beta: oauth-2025-04-20`** (les jetons OAuth ne passent pas par
  `x-api-key`).
- `libertyd` rafraîchit les jetons en arrière-plan ; l'humain ne voit jamais
  une clé.

### 2. Clé API (mode avancé / headless)
- `ANTHROPIC_API_KEY`, envoyée via l'en-tête `x-api-key`.
- Pour serveurs, CI, utilisateurs avancés.

Règle de résolution (alignée sur l'écosystème Anthropic) : variable
`ANTHROPIC_API_KEY` → `ANTHROPIC_AUTH_TOKEN` (OAuth) → profil stocké. Piège
connu : une clé API résiduelle dans l'environnement masque le profil OAuth —
`libertyd` doit le détecter et l'afficher clairement.

## Comment `libertyd` parle à Claude

- **Endpoint unique** : `POST /v1/messages` (API Messages).
- **Tool use = bus d'intents.** Les actions système de Liberty (brider un
  processus, ranger des fichiers, répondre à un mail…) sont déclarées à
  Claude comme des *outils* avec schémas JSON. Claude décide, émet des
  `tool_use` ; l'OS vérifie les capacités puis exécute. C'est exactement le
  modèle d'intents qu'on avait conçu — l'API de Claude le supporte
  nativement.
- **Thinking adaptatif** (`thinking: {type: "adaptive"}`) pour que Claude
  raisonne sur les cas ambigus et réponde vite sur les cas simples.
- **Streaming SSE** pour toute interaction visible par l'humain (la barre
  universelle affiche la réponse au fil de l'eau).
- **Prompt caching** : le prompt système de `libertyd` (philosophie
  d'autonomie, capacités accordées, outils) est stable et long → mis en
  cache (`cache_control: ephemeral`), le contexte volatil (situations du
  moment) arrive après. Latence et coût réduits massivement.
- **Le jugement calibré devient réel.** Au lieu d'un `confidence` codé en
  dur (prototype), Claude évalue chaque situation et choisit lui-même entre
  *agir*, *proposer*, ou *poser une question précise* — via des outils
  dédiés (`act`, `propose`, `ask_user`). La couche `decide()` de l'OS reste
  le filet de sécurité au-dessous.

## Statut d'implémentation

- ✅ Boucle de décision (`decide()`) avec capacités, autonomie, réversibilité.
- ✅ Trait `Brain` : backend interchangeable (`SimulatedBrain` pour le dev
  hors-ligne, `ClaudeBrain` qui construit les vraies requêtes API).
- ✅ Résolution des identifiants (clé API / jeton OAuth) + forme exacte des
  requêtes HTTP (testée unitairement).
- ⬜ Transport HTTP réel + flux OAuth complet (nécessite réseau).
- ⬜ Streaming SSE dans la barre universelle.
- ⬜ Réflexes locaux (classification Haiku / modèle embarqué).
