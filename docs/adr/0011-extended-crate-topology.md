# ADR-0011 — Topologie de crates étendue

## Statut
Accepted

## Contexte
ADR-0010 avait posé une architecture en couches (`domain/use_cases/
infrastructure`) *à l'intérieur* de chaque crate fonctionnel, en écartant
explicitement les crates transverses pour rester fidèle à ADR-0001 (un crate
= un domaine fonctionnel). Avant qu'aucun code ne soit écrit sous cette
forme, une cible bien plus large a été retenue : chaque responsabilité
transverse (bus d'événements, système de plugin, accès fichiers, LSP,
terminal, Git, hébergement de plugins tiers) devient sa propre crate au lieu
d'un sous-module. Cette décision supersede ADR-0010 sur ce point précis, et
fait évoluer ADR-0002 (event bus) et ADR-0009 (plugins statiques) quant à
l'emplacement du code, sans remettre en cause leurs principes de
fonctionnement.

## Décision
- Le workspace s'étend avec de nouvelles crates : `forge-plugin` (trait
  `Plugin`, contexte fourni aux plugins), `forge-plugin-host` (futur
  chargement de plugins tiers, vide pour l'instant), `forge-event` (bus
  d'événements, `AppEvent`), `forge-fs` (accès/observation du système de
  fichiers, partagé), `forge-lsp` (intégration Language Server Protocol),
  `forge-terminal` (terminal intégré), `forge-git` (intégration Git). Toutes
  vides au départ, remplies au fur et à mesure des besoins réels (même
  principe que les crates existantes).
- `forge-tui` est supprimé : son rôle est absorbé par `forge-ui`, qui devient
  la façade UI réelle du projet plutôt qu'un second squelette parallèle.
- `AppEvent` déménage de `forge-runtime` vers `forge-event`. Le trait
  `Plugin` et le contexte fourni aux plugins déménagent de `forge-runtime`
  vers `forge-plugin`. `forge-runtime` dépend désormais de ces deux crates
  au lieu de porter ce code lui-même — son rôle reste le même (orchestrateur
  central, lifecycle, ADR-0001), seul l'emplacement du code change.
- `forge-editor::Editor` reste un `Plugin` enregistré tel quel pour l'instant
  (import mis à jour vers `forge_plugin`) ; son passage sous la responsabilité
  de `forge-ui` et sa restructuration interne (`Document`/`Buffer` séparés)
  sont différés à une phase ultérieure de cette même migration, pas traités
  ici.
- Migration en phases plutôt qu'en un seul passage : cette ADR documente la
  topologie cible complète, mais l'implémentation avance par étapes
  reviewables (cf. `docs/ROADMAP.md` pour le détail des phases restantes).

## Conséquences
### Avantages
- Chaque responsabilité transverse a une frontière de compilation propre
  (dépendances explicites dans `Cargo.toml`), plus proche de la vision de
  conception d'origine du projet.
- Ouvre la voie à des crates réellement réutilisables indépendamment de
  `forge-editor` (ex: `forge-fs` pour un futur explorateur de fichiers,
  `forge-lsp` sans dépendre de l'éditeur).

### Inconvénients
- Beaucoup plus de crates à faire évoluer en parallèle ; plus de
  `Cargo.toml` à maintenir pour un projet qui reste, pour l'instant, à un
  stade très minimal fonctionnellement.
- `ADR-0010` est en grande partie caduque après avoir été adoptée sans qu'aucun
  code n'ait encore été écrit sous cette forme — gardée pour l'historique de
  la décision, mais son contenu ne doit plus guider l'organisation interne
  des crates existantes.
- Certaines crates (`forge-plugin-host`, `forge-fs`, `forge-lsp`,
  `forge-terminal`, `forge-git`) restent vides pendant un temps indéterminé,
  en attendant un vrai consommateur — accepté comme compromis pour ne pas
  bloquer la mise en place de la topologie cible en attendant que chaque
  besoin soit concret.
