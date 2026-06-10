# Installer Liberty dans une machine virtuelle

Liberty v0.2 s'installe **par-dessus une Debian minimale** : on garde le
noyau et les drivers mûrs de Linux, et la couche Liberty (l'esprit `libertyd`,
le shell `lish`, la config et le service) prend le contrôle au-dessus. À
terme, une image disque Liberty dédiée remplacera cette étape.

Durée totale : ~30 minutes (dont l'essentiel en installation Debian et
compilation).

## 1. Préparer la VM

N'importe quel hyperviseur fait l'affaire. Sur Windows Home, **VirtualBox**
est le plus simple (Hyper-V n'y est pas disponible) :

1. Installer [VirtualBox](https://www.virtualbox.org/wiki/Downloads).
2. Télécharger l'ISO **Debian netinst** (amd64) :
   <https://www.debian.org/download>
3. Créer une VM :
   - Type : Linux / Debian (64-bit)
   - Mémoire : **4 Go minimum** (8 Go confortable)
   - Disque : **20 Go**
   - Réseau : NAT (par défaut) — il faut un accès Internet (API Anthropic).

## 2. Installer Debian (minimale)

Démarrer la VM sur l'ISO et suivre l'installateur :

- Choisir l'installation **non graphique** (« Install »).
- Créer l'utilisateur (ex. `march`) et un mot de passe root.
- Partitionnement : « assisté — utiliser tout le disque ».
- **Étape importante — sélection des logiciels** : décocher *Debian desktop
  environment* et *GNOME* ; ne garder que **« utilitaires usuels du
  système »** (et « serveur SSH » si tu veux y accéder depuis l'hôte).

Au redémarrage, tu arrives sur une console texte : c'est exactement ce qu'on
veut. Liberty *est* l'interface.

## 3. Installer Liberty

Se connecter en **root**, puis :

```sh
apt-get update && apt-get install -y curl ca-certificates
curl -fsSL https://raw.githubusercontent.com/Alcegobe/Liberty/main/install/liberty-install.sh | sh
```

Le script :

1. installe les dépendances (git, gcc, Rust) ;
2. clone et compile Liberty (`libertyd` + `lish`, transport Claude activé) ;
3. installe la config dans `/etc/liberty/liberty.toml` ;
4. demande ta **clé API Anthropic** (créée sur
   [console.anthropic.com](https://console.anthropic.com)) et la range dans
   `/etc/liberty/anthropic.key` (mode 600) ;
5. installe et démarre le service systemd `libertyd` ;
6. déclare `lish` comme shell de connexion possible.

Options utiles :

```sh
# tout-en-un, sans question interactive, avec lish comme shell de march :
ANTHROPIC_API_KEY=sk-ant-... LIBERTY_USER=march sh liberty-install.sh

# installer une branche précise :
LIBERTY_REF=ma-branche sh liberty-install.sh
```

## 4. Vérifier que l'esprit vit

```sh
systemctl status libertyd          # le démon doit être « active (running) »
journalctl -u libertyd -f          # le suivre en direct : à chaque battement,
                                   # il observe, décide, agit, fait son bilan
libertyd --once                    # forcer un battement à la main
```

Tu verras l'esprit lire les capteurs (disque, mémoire, services, journaux),
décider, et conclure par un bilan — chaque action étant filtrée par les
capacités de `/etc/liberty/liberty.toml` et consignée dans
`/var/lib/liberty/journal.jsonl`.

## 5. Parler au système : `lish`

```sh
lish
```

```
lish — Liberty Shell
esprit : Claude (claude-fable-5) · profil : Prudent

◆ fais de la place sur le disque
  👁  observe : df -h
  👁  observe : du -sh /var/cache/apt
  ⚙  exécute : apt-get clean
✔ 312 Mo libérés (cache APT purgé, régénérable).

◆ :journal      ← tout ce que l'esprit a fait
◆ :undo         ← annule la dernière action annulable
◆ !htop         ← shell brut, sans IA
◆ exit
```

Pour faire de `lish` ton shell de connexion (la VM démarre « dans » Liberty) :

```sh
chsh -s /usr/local/bin/lish march
```

## 6. Régler l'autonomie

Édite `/etc/liberty/liberty.toml` puis `systemctl restart libertyd` :

- `profile = "prudent"` — système autonome, fichiers sur proposition,
  communication manuelle (défaut) ;
- `profile = "confiance"` — l'esprit gère tout, le réversible en silence ;
- `profile = "manuel"` — l'esprit suggère, l'humain fait.

La section `[capabilities]` est le garde-fou matériel : tout effet non listé
est refusé par l'OS, quelles que soient la confiance et l'autonomie. Le
réseau n'est **jamais** accordé par défaut.

## Mises à jour

```sh
curl -fsSL https://raw.githubusercontent.com/Alcegobe/Liberty/main/install/liberty-install.sh | sh
```

Le script ré-utilise `/opt/liberty/src`, recompile et redémarre le service.
La config et la clé existantes ne sont pas touchées.

## Dépannage

| Symptôme | Cause probable | Remède |
|---|---|---|
| `libertyd` inactif, « aucun compte Anthropic lié » | clé absente/vide | remplir `/etc/liberty/anthropic.key`, `systemctl restart libertyd` |
| « Connexion Anthropic échouée » | pas d'Internet dans la VM | vérifier le NAT, `ping api.anthropic.com` |
| compilation très lente | VM à 1 CPU | donner 2-4 vCPU à la VM |
| `lish` répond « compilé sans transport réseau » | build sans feature | relancer le script d'installation |
