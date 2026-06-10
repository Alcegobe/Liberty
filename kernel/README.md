# kernel/

Configuration du noyau **Linux minimal** (LTS) qui sert de socle matériel à
Liberty, et artefacts de boot (UEFI).

À terme, ce dossier contiendra :
- la configuration de build du noyau (config minimaliste, modules requis) ;
- les scripts de construction de l'image noyau ;
- éventuellement, des modules noyau spécifiques à Liberty (Rust).

> Rappel : Liberty ne réécrit pas un noyau de zéro. Le noyau est un détail
> d'implémentation ; l'identité de Liberty vit dans le userland (`userland/`,
> `services/`).
