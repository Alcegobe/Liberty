# Roadmap — Liberty OS

Plan par phases. Chaque phase produit quelque chose de **démontrable**, pas
seulement du code. On avance par petits jalons vérifiables.

> Changement de stratégie (juin 2026) : on construit **la couche d'abord,
> l'image ensuite**. L'identité de Liberty, c'est son esprit (`libertyd`) et
> son shell (`lish`) — ils tournent dès aujourd'hui sur une Debian minimale
> en VM. L'image disque dédiée, le compositeur et le reste viennent
> s'empiler dessous et autour, sans bloquer l'essentiel.

---

## Phase 0 — Fondations *(fait)*
- [x] Vision, architecture et roadmap rédigées.
- [x] Structure du dépôt.
- [x] Pivot : Claude (Fable 5, puis chaque modèle plus capable) est l'esprit.

## Phase 1 — L'esprit vivant *(fait — v0.2)*
- [x] **Boucle de décision** : capacités, autonomie par domaine, règle
      réversible/local, seuils de confiance (testée).
- [x] **Capteurs locaux** : charge, mémoire, disques, processus, services en
      échec, journaux d'erreurs → rapport de situation minimisé.
- [x] **Boucle agentique réelle** : outils `observe` (lecture seule, liste
      blanche) / `act` (passé par `decide()`) / `ask_user` / `done`,
      multi-tours, thinking adaptatif, prompt caching.
- [x] **Exécuteur journalisé** : toute action consignée
      (`/var/lib/liberty/journal.jsonl`), commande d'annulation conservée.
- [x] **`libertyd --daemon`** : battement de cœur autonome périodique.
- [x] **`lish`** : le shell en langage naturel (validation interactive,
      `:journal`, `:undo`, `!` pour le shell brut).
- [x] **Config système** : `/etc/liberty/liberty.toml` (profil, capacités,
      modèle, rythme).
- [x] **Politique de modèle** : découverte à l'exécution du Claude le plus
      capable accessible au compte.

**Démontrable :** une VM Debian devient Liberty en une commande ; l'esprit
tourne en service, le shell répond en langage naturel.
→ [`docs/INSTALL.md`](INSTALL.md)

## Phase 2 — L'esprit digne de confiance *(en cours)*
- [ ] Flux OAuth complet (« se connecter avec son compte Anthropic »).
- [ ] File de propositions consultable/validable depuis `lish`
      (`:pending`, approuver/refuser ce que le démon a mis en attente).
- [ ] Fenêtre d'annulation effective pour les actions externes (envoi différé).
- [ ] Modèle de menace prompt-injection (`docs/SECURITY.md`) : les sorties de
      capteurs sont des *données*, jamais des instructions.
- [ ] Mémoire persistante de l'esprit entre battements (contexte machine,
      préférences apprises) — locale et chiffrée.
- [ ] Réflexes locaux : pré-tri Haiku / heuristiques avant d'appeler l'esprit
      (coût et latence).
- [ ] CI : build + tests à chaque commit.

**Démontrable :** le démon tourne des jours entiers, ses propositions
s'examinent et s'approuvent depuis `lish`, rien d'irréversible ne part seul.

## Phase 3 — L'image Liberty
- [ ] Image disque bootable (UEFI) : Linux minimal + init léger + libertyd +
      lish en autologin — plus de Debian à installer.
- [ ] `liberty-image build` reproductible (mkosi ou équivalent).
- [ ] Mises à jour atomiques avec rollback.

**Démontrable :** `make image && make run` → QEMU boote directement dans
Liberty.

## Phase 4 — Les yeux et les mains élargis
- [ ] Bus d'intents pour applications : une app déclare ses actions,
      `libertyd` les expose à l'esprit comme outils.
- [ ] Domaines supplémentaires : réseau (wifi), paquets, courriel réel.
- [ ] Streaming SSE dans `lish` (réponse au fil de l'eau).

**Démontrable :** « trie ces photos par date » exécuté via une app dédiée.

## Phase 5 — Le visage
- [ ] Compositeur Wayland minimal (Rust, piste smithay).
- [ ] Shell graphique : barre universelle en langage naturel, fenêtres
      épurées — `lish` devient une surface, pas seulement un terminal.

**Démontrable :** un bureau sobre où la barre universelle remplace menus et
icônes.

## Phase 6+ — Vers l'utilisable au quotidien
- Multi-utilisateur, trousseau chiffré, sandbox par défaut.
- Audio, énergie, multi-écrans.
- Écosystème d'applications natives parlant le bus d'intents.

---

## Comment on travaille

- **Petits jalons vérifiables.** Chaque tâche doit tourner pour de vrai.
- **Documenter en avançant.** Les décisions importantes → `docs/`.
- **La sécurité est invariante.** Quelle que soit la phase : capacités
  d'abord, journal toujours, l'OS au-dessus du modèle.
