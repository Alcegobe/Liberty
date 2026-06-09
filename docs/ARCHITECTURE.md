# Architecture — Liberty OS

> Document vivant. Les décisions ici sont des points de départ raisonnés,
> destinés à être challengés au fil du projet.

## Vue d'ensemble en couches

```
┌─────────────────────────────────────────────────────────────┐
│  Applications (sandboxées, capacités explicites)             │
├─────────────────────────────────────────────────────────────┤
│  Shell Liberty   │  Compositeur Wayland  │  Apps système     │
│  (UI + NL)       │  (rendu, fenêtres)    │  (fichiers, …)    │
├─────────────────────────────────────────────────────────────┤
│  libertyd  — couche IA système (inférence locale, intents,   │
│              orchestration d'actions, mémoire/contexte)      │
├─────────────────────────────────────────────────────────────┤
│  Services système Liberty (Rust) :                           │
│  init/superviseur · paquets · réseau · sécurité/capacités    │
├─────────────────────────────────────────────────────────────┤
│  Noyau Linux minimal (LTS) — drivers, scheduler, mémoire     │
├─────────────────────────────────────────────────────────────┤
│  Matériel : x86_64 / ARM64 · CPU · GPU · NPU · NVMe          │
└─────────────────────────────────────────────────────────────┘
```

## Principes directeurs

1. **Le noyau est un détail d'implémentation.** On utilise Linux pour le
   matériel. Tout ce qui définit Liberty vit au-dessus. On garde une
   frontière nette pour pouvoir, un jour, remplacer/compléter le noyau.
2. **Capabilities, pas permissions globales.** Chaque processus reçoit des
   capacités explicites (accès fichier X, réseau, caméra…). Rien par défaut.
3. **Tout est inspectable par l'IA.** État système, fichiers, actions des
   apps exposés via une API structurée que `libertyd` peut lire et composer.
4. **Immuabilité.** Système de base immuable + couches utilisateur. Mises à
   jour atomiques avec rollback (inspiré de NixOS / ostree).

## Composants

### Noyau (`kernel/`)
- Config Linux minimaliste, modules strictement nécessaires.
- Phase ultérieure possible : modules Rust (déjà supporté en amont).
- Boot : UEFI + un bootloader simple (systemd-boot ou un loader maison Rust).

### Init / superviseur (`services/`)
- Process 1 maison en Rust : démarrage parallèle, supervision, sockets
  d'activation. Léger, rapide, prévisible.

### `libertyd` — couche IA système (`services/libertyd/`)
- **Runtime d'inférence local** : exécute des modèles (LLM/SLM, vision) sur
  CPU/GPU/NPU. Backend interchangeable (p. ex. llama.cpp/ONNX/candle).
- **Bus d'intents** : les apps déclarent des outils/actions ; `libertyd`
  les découvre et les compose pour répondre aux requêtes en langage naturel.
- **Garde-fou de capacités** : toute action IA passe par le contrôle de
  capacités et, si nécessaire, une confirmation utilisateur.
- **Mémoire/contexte** : stockage local chiffré du contexte utilisateur.
- **Confidentialité** : local par défaut ; tout appel réseau est explicite,
  visible et révocable.

### Compositeur (`userland/compositor/`)
- Compositeur Wayland en Rust (piste : smithay).
- Rendu accéléré GPU, vsync, HDR à terme. Esthétique épurée.

### Shell Liberty (`userland/shell/`)
- L'UI principale : lanceur, barre, gestion de fenêtres.
- **Entrée en langage naturel de première classe** : une barre universelle
  où l'on tape/dit une intention, traduite en actions par `libertyd`.

### Runtime IA (`userland/ai-runtime/`)
- Bibliothèques partagées + SDK pour que les apps tierces exposent des
  actions et consomment l'inférence locale.

### Gestionnaire de paquets
- Déclaratif et atomique. Description du système en un fichier ; build
  reproductible ; rollback. Sandbox par défaut pour les apps.

## Choix technologiques (et pourquoi)

| Décision            | Choix              | Raison                              |
|---------------------|--------------------|-------------------------------------|
| Langage userland    | **Rust**           | Sûreté mémoire, perf, écosystème    |
| Noyau               | **Linux LTS min.** | Drivers gratuits, maturité          |
| Affichage           | **Wayland**        | Moderne, sécurisé, pas d'héritage X |
| Cibles              | **x86_64, ARM64**  | PC actuels + ARM (efficacité)       |
| Test/dev            | **QEMU/KVM**       | Itération rapide sans matériel      |
| Inférence locale    | Backend modulaire  | Suivre l'état de l'art (candle/ONNX)|

## Questions ouvertes (à trancher en chemin)

- Format de paquet et modèle de build exact (inspiration Nix vs ostree).
- Backend d'inférence par défaut et stratégie multi-accélérateur.
- Modèle de sécurité fin : implémentation des capacités (jetons, LSM…).
- Stratégie de compatibilité applicative (apps Linux existantes via couche
  de compat ? ou écosystème natif uniquement ?).
