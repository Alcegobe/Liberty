# services/

Services système de Liberty, écrits en **Rust**.

- `libertyd/` — la couche IA système (runtime d'inférence local, bus
  d'intents, orchestration d'actions sous contrôle de capacités).
- (à venir) init/superviseur (PID 1), logs, IPC, réseau, sécurité.

Voir [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md).
