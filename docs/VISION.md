# Vision — Liberty OS

## Le problème

Les OS dominants (Windows, macOS) portent un héritage de 30+ ans. Leur cœur
a été conçu pour un monde sans IA, sans souci de vie privée, et avant le
matériel moderne (NPU, SSD NVMe, multi-cœurs massifs). L'IA y est greffée par
le dessus, app par app, de façon incohérente et souvent dépendante du cloud.

## La thèse de Liberty

Un OS conçu **maintenant** doit avoir l'IA dans son ADN. Pas un assistant
dans un coin : une **couche d'intelligence système** que toute application
peut invoquer, et qui peut agir sur le système au nom de l'utilisateur.

Mais Liberty va plus loin qu'« une IA qui obéit ». Le cœur de sa vision, c'est
**l'inversion** : l'IA mène la vie numérique et ne sollicite l'humain que
lorsqu'elle en a besoin. Tu ne poses plus les requêtes — tu **réponds** aux
questions que l'IA juge importantes. Voir [`INVERSION.md`](INVERSION.md).

## Les quatre promesses à l'humain (la boussole)

Avant la technique, ce que l'humain gagne vraiment. Tout le reste découle de
ces quatre désirs :

1. **Du temps.** La corvée (ranger, configurer, chercher, répéter, trier ses
   mails) est déléguée — et souvent faite en silence.
2. **De la vie privée.** L'IA est **locale par défaut** ; ta vie ne part pas
   dans le cloud.
3. **De la simplicité.** Plus besoin de savoir *comment* faire, ni de bien
   « prompter » : tu dis ton but, ou l'IA anticipe et te pose une question
   précise.
4. **Du contrôle.** L'IA peut agir, et même agir seule, **sans déborder** :
   effets déclarés + capacités + réversibilité + transparence consultable.
   Tu règles son niveau d'autonomie. Voir [`AUTONOMY.md`](AUTONOMY.md).

> Personne ne se réveille en voulant « un OS AI-first ». On veut récupérer du
> temps, garder sa vie privée, ne plus se battre avec sa machine, et rester
> maître de ce qui se passe chez soi. L'IA-first est la *plomberie* au service
> de ces quatre désirs. C'est la boussole : si un choix technique ne sert pas
> l'un d'eux, on le questionne.

### Trois piliers

1. **AI-first**
   - Un service système, `libertyd`, expose un runtime d'inférence local.
   - L'utilisateur s'adresse au système en langage naturel (« range mes
     téléchargements par projet », « réduis la conso batterie »).
   - Les applications déclarent des *actions* que l'IA peut composer
     (intents/outils), avec un contrôle de capacités strict.
   - **Local par défaut.** Le cloud est opt-in, explicite, et jamais requis.

2. **Épuré**
   - Une seule bonne façon de faire chaque chose.
   - Interface sobre, cohérente, sans bloatware.
   - Système de fichiers lisible par l'humain et par l'IA.

3. **Ultra-optimisé**
   - Userland en Rust : sûreté mémoire sans GC, performances proches du C.
   - Démarrage en secondes, empreinte RAM minimale au repos.
   - Exploitation native du matériel moderne (NPU/GPU pour l'inférence).

## Ce que Liberty n'est PAS

- Pas une réécriture de noyau de zéro (irréaliste, et inutile : Linux est
  excellent et libre).
- Pas une distribution Linux de plus avec un thème. La différence est
  architecturale : userland propre + IA système native.
- Pas un produit cloud déguisé. La vie privée est un principe de conception.

## Public cible (au départ)

- Développeurs et early adopters qui veulent un poste de travail rapide,
  scriptable et piloté par IA locale.
- Élargissement progressif vers le grand public à mesure que l'écosystème
  d'applications mûrit.

## Mesures de succès

- **Démarrage** : bureau utilisable en < 5 s sur matériel courant.
- **Mémoire** : < 400 Mo de RAM au repos (hors modèle IA chargé).
- **IA locale** : réponse d'un assistant local en < 1 s pour les tâches
  courantes sur un NPU/GPU grand public.
- **Cohérence** : zéro « panneau de configuration hérité ». Une seule UI.
