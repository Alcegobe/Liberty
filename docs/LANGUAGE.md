# Le langage Lib — un langage pensé pour l'IA

> Statut : 🌱 conception + prototype. Voir `tools/lib-interpreter/` pour un
> interpréteur minimal qui exécute déjà du Lib.

## Pourquoi un nouveau langage ?

Les langages actuels ont été conçus **pour des humains tapant du texte**. Lib
est conçu pour un monde où **l'IA et l'humain écrivent le code ensemble**, sur
un OS où l'IA est un service système de première classe (`libertyd`).

Lib n'essaie pas de remplacer Rust. Rust reste le langage du **cœur système**
(noyau-socle, compositeur, services bas niveau). Lib est le langage du
**dessus** : applications, automatisations, et surtout les **actions** que
l'IA compose pour l'utilisateur.

## Ce qui rend un langage difficile pour une IA (et que Lib corrige)

Une IA génère le code **token par token, sans revenir en arrière**. Ce qui la
fait échouer, et la réponse de Lib :

| Problème des langages actuels | Réponse de Lib |
|---|---|
| Dépendances lointaines (variable déclarée 400 lignes plus haut) | **Localité** : tout ce qu'il faut pour comprendre un bloc est dans le bloc |
| Magie implicite (coercions, macros, effets cachés) | **Explicite** : aucun effet caché ; ce que le code touche est déclaré |
| Plusieurs façons d'écrire la même chose | **Forme canonique unique** par construction |
| On ne sait pas tout de suite si c'est faux | **Vérifiable en direct** : types stricts, retour immédiat |
| Le texte autorise les fautes de frappe / syntaxe | **Cœur structuré** : le code *est* un arbre typé ; le texte n'est qu'une projection |

## Les quatre principes de conception

### 1. Effets déclarés (et vérifiés par l'OS)
Toute action déclare ce qu'elle **touche** (fichiers, réseau, caméra…). L'OS
(`libertyd` + couche de capacités) **refuse** ce qui n'a pas été autorisé.
C'est la sécurité quand c'est une IA qui écrit le code qui s'exécute chez toi.

```lib
action ranger {
  touche "fichiers:~/Téléchargements"   # déclaré → vérifié → autorisé
  classe telechargements par extension
}
```

### 2. Localité totale
Pas d'import implicite, pas d'état global caché. Un bloc se comprend seul.

### 3. Primitives AI-native
Des opérations de haut niveau (`classe … par …`, `résume`, `trie par …`) que
`libertyd` peut exécuter intelligemment, plutôt que de réimplémenter à la main.

### 4. Cœur structuré, projection texte
À terme, la représentation canonique de Lib n'est pas du texte mais une
**structure typée**. L'IA émet la structure ; l'humain lit/édite une
projection texte lisible (comme ci-dessus). Conséquence : les erreurs de
syntaxe et les fautes de frappe deviennent **impossibles**.

> Le prototype actuel utilise du texte comme entrée (étape pragmatique). La
> structure typée est la cible à mesure que le langage mûrit.

## Aperçu de la syntaxe (projection texte)

```lib
permet "fichiers:~/Téléchargements"        # accorder une capacité

soit fichiers = ["a.pdf", "b.png", "c.pdf"] # une liaison locale

affiche "Bonjour " + "Liberty"             # afficher

pour chaque f dans fichiers {              # itérer
  affiche "  - " + f
}

action ranger {                            # une action à effets déclarés
  touche "fichiers:~/Téléchargements"
  classe fichiers par extension            # primitive AI-native
}
```

## Feuille de route du langage

1. **Prototype** (fait) : interpréteur arbre-syntaxique en Rust, sous-ensemble
   minimal (liaisons, itération, actions à effets, primitive `classe`).
2. **Typage statique** : types stricts, vérification des effets à la
   compilation.
3. **Représentation structurée** : l'AST typé devient la forme canonique ;
   le texte devient une projection éditable.
4. **Backend optimisant** : compilation vers LLVM/Cranelift pour la vitesse
   (on réutilise 20 ans d'optimisations plutôt que de les réécrire).
5. **Co-conception avec l'IA** : un modèle local spécialisé (fine-tuné) qui
   écrit/lit Lib nativement. Voir `docs/AI.md`.
