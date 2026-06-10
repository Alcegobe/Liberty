# Le modèle d'autonomie de Liberty

> Comment l'IA agit seule, quand elle te consulte, et comment tu règles tout
> ça. Complément de [`INVERSION.md`](INVERSION.md).

## Principe : l'autonomie est un curseur, pas un interrupteur

On ne choisit pas *pour tout le monde* entre « ça demande toujours » et « ça
fait toujours ». Le bon niveau dépend **de l'action** et **de la personne**.
Donc l'autonomie est **réglable**, et réglable **par domaine**.

### Trois niveaux, posables sur chaque type d'action

| Niveau | Comportement | Exemples |
|---|---|---|
| 🟢 **Autonome (silencieux)** | L'OS détecte *et* résout seul. L'humain n'est pas dérangé. | PC qui chauffe, cache à vider, index à reconstruire, correctif de sécurité interne, doublons |
| 🟡 **Propose** | L'IA prépare, l'humain valide d'un mot. | Réglage important, mail sensible, opération de masse inhabituelle |
| 🔴 **Manuel** | L'humain fait, l'IA assiste. | Ce que l'utilisateur veut garder sous sa main |

Un **curseur global** (« prudent » ↔ « gère pour moi ») ne fait que déplacer
les valeurs par défaut de chaque domaine. L'utilisateur peut ensuite ajuster
domaine par domaine.

## La règle : qu'est-ce qui peut être 100 % silencieux ?

Deux questions suffisent :

1. **C'est réversible ?** (peut-on annuler proprement ?)
2. **Ça reste chez moi ?** (ou ça touche l'extérieur / d'autres personnes /
   de l'argent ?)

- **Réversible + local** → autonomie silencieuse, sans hésiter.
- **Irréversible**, **ou externe**, **ou affectant autrui** → l'IA *choisit*
  de consulter (jugement calibré, cf. ci-dessous).

## Le principe de conception le plus puissant

> Au lieu d'**interdire** d'automatiser les choses irréversibles, on **rend
> les choses réversibles** — et alors on peut les automatiser sans risque.

| Action | Rendue sûre par… | Résultat |
|---|---|---|
| Supprimer un fichier | Versionnage / corbeille longue durée | Auto, silencieux ✅ |
| Ranger / renommer | Annulable | Auto, silencieux ✅ |
| Répondre à un mail anodin | Style appris + **délai d'envoi annulable** | Auto, fenêtre de rattrapage |
| Mail sensible / paiement / envoi à un tiers | Touche autrui ou l'argent | L'IA consulte |

Maximiser la réversibilité, c'est maximiser ce qu'on peut automatiser en paix.

## La seule limite honnête

Ce qui **affecte d'autres personnes** ou est **vraiment irrécupérable** te
brûle exactement une fois — et ça suffit à détruire la confiance. La réponse
n'est pas une règle rigide, mais le **jugement calibré** de l'IA : elle évalue
sa confiance et l'enjeu, agit seule quand c'est sûr, remonte vers toi sinon.
Ce jugement s'affine et reste réglable.

## Ce qui rend tout ça SÛR (et pas flippant)

L'autonomie élevée est sûre **grâce** à l'architecture, pas malgré elle :

- **Capacités + effets déclarés** : une IA autonome est *bornée*. Elle ne peut
  pas physiquement déborder de ce qui lui est accordé, même en agissant seule.
  (Voir [`ARCHITECTURE.md`](ARCHITECTURE.md) et [`LANGUAGE.md`](LANGUAGE.md).)
- **Réversibilité par conception** : versionnage, corbeille longue, délais
  d'envoi annulables.
- **Transparence consultable** : **silencieux ≠ caché.** Même en agissant
  seule, l'IA tient un journal clair, ouvrable à tout moment — *« aujourd'hui :
  bridé un processus emballé, libéré 3 Go, appliqué un correctif. »* On n'est
  pas dérangé, mais rien n'est dissimulé, et tout est annulable.

## L'autonomie se gagne

L'IA remarque les régularités : *« tu as approuvé ce type d'action 20 fois —
je m'en occupe seule désormais ? »*. Le curseur monte au rythme de la
confiance, catégorie par catégorie.
