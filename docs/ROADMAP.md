# Roadmap — Liberty OS

Plan par phases. Chaque phase produit quelque chose de **démontrable** (qui
boote/tourne sous QEMU), pas seulement du code. On avance par petits jalons
vérifiables.

> Réalisme : c'est un projet de très long terme. L'objectif des premières
> phases n'est pas un OS « fini » mais une **boucle de développement solide**
> et une **identité technique claire**.

---

## Phase 0 — Fondations *(en cours)*
- [x] Vision, architecture et roadmap rédigées.
- [x] Structure du dépôt.
- [ ] Choix de licence + `CONTRIBUTING`.
- [ ] Environnement de build reproductible (toolchain Rust, QEMU, scripts).
- [ ] CI : build + lint à chaque commit.

**Démontrable :** `make setup && make doctor` valide l'environnement.

## Phase 1 — « Hello, Liberty » bootable
- [ ] Image disque minimale qui boote sous QEMU (UEFI).
- [ ] Noyau Linux minimal configuré + init maison qui affiche une bannière.
- [ ] Console texte fonctionnelle.

**Démontrable :** `make run` → la machine boote sur Liberty en console.

## Phase 2 — Userland Rust de base
- [ ] Init/superviseur en Rust (PID 1) : démarrage parallèle + supervision.
- [ ] Quelques services système de base (logs, IPC, device manager léger).
- [ ] Premier outil en ligne de commande « libertyctl ».

**Démontrable :** services supervisés, redémarrage propre d'un service tué.

## Phase 3 — Affichage et shell
- [ ] Compositeur Wayland minimal (une fenêtre, un curseur, du rendu GPU).
- [ ] Shell Liberty : barre universelle + lanceur d'apps.

**Démontrable :** bureau épuré qui s'affiche et lance une app simple.

## Phase 4 — `libertyd`, la couche IA
- [x] **Prototype de la boucle de décision** (inversion, niveaux d'autonomie,
      règle réversible/local, capacités, jugement calibré, journal). Voir
      `services/libertyd/`.
- [ ] Service `libertyd` avec runtime d'inférence local (modèle léger).
- [ ] Barre universelle → langage naturel → action système simple
      (ex. « ouvre le dossier Documents », « règle la luminosité »).
- [ ] Modèle de capacités appliqué aux actions IA (réel, plus simulé).

**Démontrable :** `cargo run -p libertyd` montre déjà l'IA qui initie, décide
selon l'autonomie, et se fait borner par l'OS. Ensuite : on tape une phrase,
le système agit — 100 % local.

## Phase 5 — Bus d'intents pour applications
- [ ] SDK : une app déclare des actions ; `libertyd` les découvre/compose.
- [ ] App de démonstration (gestionnaire de fichiers) pilotable par IA.

**Démontrable :** « trie ces photos par date » exécuté via l'app.

## Phase 6 — Paquets, mises à jour atomiques, sécurité
- [ ] Gestionnaire de paquets déclaratif + rollback atomique.
- [ ] Sandbox des apps par défaut.

**Démontrable :** installer/supprimer une app et revenir en arrière sans
casser le système.

## Phase 7+ — Vers l'utilisable au quotidien
- Réseau (wifi/ethernet) via UI épurée.
- Audio, énergie, multi-écrans.
- Écosystème d'applications natives.
- Optionnel : couche de compatibilité pour apps Linux existantes.

---

## Comment on travaille

- **Petits jalons vérifiables.** Chaque tâche doit booter/tourner.
- **Documenter en avançant.** Les décisions importantes → `docs/`.
- **Mesurer.** Démarrage, RAM, latence IA suivis dès que possible.
- **Itérer l'architecture.** Rien n'est gravé : on challenge les choix.
