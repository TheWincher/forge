# Roadmap

Document vivant, complémentaire de CLAUDE.md : CLAUDE.md décrit l'état actuel
et les décisions déjà prises, ce fichier liste les jalons envisagés plus loin.
À tenir à jour au même rythme que CLAUDE.md — un jalon terminé se déplace vers
CLAUDE.md (et un ADR si la décision associée est significative), et disparaît
d'ici.

Les jalons ci-dessous sont un premier jet, pas figé : l'ordre et le contenu
sont à discuter/réordonner.

## État actuel (résumé — détails dans CLAUDE.md)

Runtime fonctionnel (lifecycle, signaux, config TOML, workspace optionnel,
plugins statiques). `forge-editor::Editor` est le premier `Plugin` réel,
toujours à plat (`Buffer{path, content}`), rien ne l'affiche encore — pas de
UI, pas de déclenchement automatique.

Migration en cours vers la topologie de crates étendue (ADR-0011) : chaque
responsabilité transverse devient sa propre crate plutôt qu'un sous-module.
Cette migration remplace la séquence de jalons qui existait avant
(TUI/listing/buffers multiples) — les jalons ci-dessous sont réordonnés en
conséquence : d'abord finir la migration de topologie, ensuite reprendre les
fonctionnalités.

## Phase 1 — Déplacement d'AppEvent, Plugin et RuntimeContext (en cours)

Scaffolding des crates vides fait (`forge-plugin-host`, `forge-fs`,
`forge-lsp`, `forge-terminal`, `forge-git`), `forge-tui` supprimé. Reste :
- `AppEvent` : `forge-runtime/src/event.rs` → `forge-event`.
- `Plugin` + `RuntimeContext` : `forge-runtime/src/{plugin,context}.rs` →
  `forge-plugin`.
- `forge-runtime` et `forge-editor` mis à jour pour dépendre des nouvelles
  crates au lieu de porter ce code ou de dépendre de `forge-runtime` pour
  `Plugin`.

`Editor` reste un `Plugin` enregistré comme aujourd'hui — pas de changement
de rôle dans cette phase.

## Phase 2 — forge-ui devient réel, Editor change de rôle

`forge-ui` (aujourd'hui un squelette vide) devient la façade UI réelle du
projet. `Editor` cesse d'être un `Plugin` enregistré directement par
`Runtime` et devient un composant possédé par `forge-ui`. Implique de
retirer `impl Plugin for Editor` et l'appel
`runtime.register_plugin(Box::new(Editor::new()))` dans `main.rs`. C'est le
jalon qui referme la boucle Runtime → forge-ui → Editor → quelque chose de
visible (remplace l'ancien "M1 — TUI", devenu obsolète avec la suppression
de `forge-tui`).

## Phase 3 — Restructuration du domaine forge-editor

`forge-editor` passe de `Buffer{path, content}` à `document.rs`/`buffer.rs`
séparés, avec `Cursor`/`Selection` en structure de données (pas de
comportement tant qu'aucune édition interactive n'en a besoin). Dépend
d'avoir `forge-ui` en place (Phase 2) pour avoir un consommateur réel.

## Phase 4 — Listing de fichiers (forge-fs / forge-workspace)

`Workspace` ne porte aujourd'hui que sa racine (ADR-0008, volontairement
minimal). `forge-fs` (actuellement vide) portera l'accès/observation
générique au système de fichiers ; `forge-workspace` s'appuiera dessus pour
lister des fichiers une fois qu'un consommateur concret en a besoin (arbre de
fichiers dans `forge-ui`, sélection du fichier à ouvrir).

## Phase 5 — Documents multiples dans Editor

Un seul document actif à la fois pour l'instant (décision v1 volontaire). Le
domaine restructuré en Phase 3 sera pensé pour en porter plusieurs, mais
`Editor` n'en exposera qu'un jusqu'à ce qu'un vrai usage (plusieurs fichiers
ouverts dans `forge-ui`) en justifie le besoin.

## Phase 6 — Système de commandes (forge-command)

Dispatch de commandes/raccourcis clavier vers des actions (ouvrir un fichier,
etc.). Dépend d'avoir une UI réelle (Phase 2) pour avoir quelque chose à
commander.

## Non planifié / sans direction définie

- `forge-core` : toujours un squelette vide, son rôle (types partagés entre
  crates ?) n'a pas encore été défini.
- `forge-lsp` : intégration Language Server Protocol (diagnostics,
  autocomplétion, hover, go-to-definition). Crate scaffoldée (ADR-0011),
  vide. Dépend d'avoir un éditeur multi-buffers fonctionnel (Phase 5) avant
  d'avoir un vrai besoin.
- `forge-terminal` : terminal intégré. Crate scaffoldée (ADR-0011), vide.
  Dépend d'une UI réelle (Phase 2).
- `forge-git` : intégration Git (statut, diff, blame) dans l'éditeur/
  explorateur. Crate scaffoldée (ADR-0011), vide, aucune réflexion de
  conception encore.
- `forge-plugin-host` : chargement de plugins tiers (hors du binaire).
  Crate scaffoldée (ADR-0011), vide — ADR-0009 (plugins statiques
  uniquement) reste en vigueur tant qu'aucun besoin concret n'apparaît.
- `forge-project` : notion de projet distincte du workspace (config projet,
  détection de type de projet, etc.) — périmètre exact pas encore défini,
  pourrait à terme fusionner avec `forge-workspace`.
- `forge-theme` / `forge-settings` : thèmes et réglages utilisateur
  persistants, au-delà de la config déjà couverte par `forge-config`
  (ADR-0007). Pas de besoin concret tant qu'il n'y a pas de rendu à styler.
- Cycle de vie des plugins au-delà de `init()` : `Plugin` n'a qu'un hook
  `init()` (ADR-0009, volontairement minimal). Un hook `shutdown()` ou un
  accès aux événements du bus (`AppEvent`, désormais dans `forge-event`) ne
  se justifiera que lorsqu'un second plugin réel existera avec un vrai
  besoin de coordination.
