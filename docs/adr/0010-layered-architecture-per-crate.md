# ADR-0010 — Architecture en couches par crate

## Statut
Superseded by ADR-0011 — aucun code n'avait encore été écrit sous cette
forme quand la décision d'aller vers une topologie de crates étendue a été
prise ; le principe "pas de crates transverses" est explicitement abandonné,
voir ADR-0011.

## Contexte
La conception initiale de Forge, esquissée avec ChatGPT avant de démarrer
l'implémentation avec Claude Code, envisageait une séparation stricte
`domain / use-cases / infrastructure / ui / application`, avec un domaine
riche (`Buffer`, `Cursor`, `Selection`, `Document`) et des services dédiés.
L'implémentation réelle avait démarré de façon volontairement minimale
(`forge-editor::Editor` à plat, un seul `Buffer{path, content}`), pour éviter
de designer avant d'en avoir besoin. Décision est prise de revenir à la
vision initiale : adopter la séparation en couches et le domaine riche,
en commençant par `forge-editor`.

## Décision
- Chaque crate fonctionnel (`forge-editor` en premier, `forge-workspace`
  ensuite) s'organise en sous-modules `domain/` (types métier purs, aucune
  I/O), `use_cases/` (orchestration métier, ex: `open_document`) et
  `infrastructure/` (frontières I/O réelles, ex: lecture disque). La racine
  du crate (`lib.rs`) devient la couche "application" : elle assemble ces
  couches et expose l'API publique du crate, dont l'implémentation du trait
  `Plugin` (ADR-0009).
- **Pas de crates transverses** (`forge-domain`, `forge-application`...) comme
  le schéma ChatGPT d'origine le suggérait : ADR-0001 (runtime propriétaire,
  un crate = un domaine fonctionnel) reste en vigueur. Les couches
  s'appliquent à l'intérieur de chaque crate, pas à travers tout le
  workspace.
- **Pas de trait/port pour les frontières I/O** (pas d'injection de
  dépendance formelle) : les tests existants dans le projet utilisent déjà
  de vrais fichiers temporaires plutôt que des mocks (`forge-config`,
  `forge-workspace`) ; la séparation en modules suffit pour la lisibilité et
  laisse la porte ouverte à une abstraction plus tard si un vrai besoin
  apparaît (VFS, accès distant...).
- Le domaine de `forge-editor` s'enrichit : `Document` remplace `Buffer` et
  porte désormais un `Cursor` et une `Selection` optionnelle. Leur
  comportement (déplacement, extension...) n'est **pas** implémenté
  maintenant — seulement la structure de données — en attendant un vrai
  besoin (édition interactive).

## Conséquences
### Avantages
- Cohérent avec la vision de conception d'origine du projet.
- Sépare clairement logique métier pure et frontières I/O, sans complexifier
  les tests (pas de mocks à maintenir).
- Chemin clair pour ajouter des cas d'usage futurs (édition, sélection
  multiple...) sans toucher aux frontières I/O ni inversement.

### Inconvénients
- Plus de fichiers/modules pour une fonctionnalité équivalente à ce qui
  existait avant (un seul `lib.rs`).
- `Cursor`/`Selection` existent en structure sans comportement pour l'instant
  — du code qui ne sert à rien tant qu'aucun cas d'usage ne les manipule,
  accepté comme compromis pour rester fidèle à la vision d'origine.
