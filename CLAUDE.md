## Mode de collaboration

Je veux apprendre en codant moi-même, pas que le code soit écrit à ma place.
Pour toute nouvelle fonctionnalité ou modification :

1. Explique l'approche et donne-moi des indications (structures, signatures,
   pièges à éviter) — mais ne montre pas le code complet.
2. Laisse-moi écrire le code moi-même.
3. Une fois que j'ai fait une implémentation, fais une review : ce qui est
   bon, ce qui pourrait être amélioré, les erreurs éventuelles.
4. Ne réécris pas mon code à ma place sauf si je le demande explicitement.

Exception : les corrections de bugs bloquants ou les questions ponctuelles
de syntaxe peuvent recevoir une réponse directe.

# Forge — Contexte projet

Forge est un IDE écrit en Rust, organisé en workspace Cargo multi-crates.
Ce fichier résume l'architecture actuelle et les décisions déjà prises, pour éviter de les reproposer ou de les remettre en question sans raison.

## Structure du workspace

```
forge/
├── Cargo.toml              # workspace, members = ["crates/*"]
└── crates/
    ├── forge/               # binaire principal (main.rs)
    ├── forge-cli/
    ├── forge-command/
    ├── forge-config/        # config de l'application (Config)
    ├── forge-core/
    ├── forge-editor/
    ├── forge-event/         # bus d'événements (AppEvent)
    ├── forge-fs/            # accès/observation du système de fichiers (vide)
    ├── forge-git/           # intégration Git (vide)
    ├── forge-lsp/           # intégration LSP (vide)
    ├── forge-plugin/        # trait Plugin + contexte fourni aux plugins
    ├── forge-plugin-host/   # chargement de plugins tiers (vide)
    ├── forge-runtime/       # coeur du runtime applicatif
    ├── forge-terminal/      # terminal intégré (vide)
    ├── forge-ui/
    └── forge-workspace/
```

Topologie étendue (ADR-0011) : chaque responsabilité transverse (bus
d'événements, système de plugin, accès fichiers, LSP, terminal, Git,
hébergement de plugins tiers) a sa propre crate plutôt qu'un sous-module.
Migration en cours par phases (voir `docs/ROADMAP.md`). `forge-tui` a été
supprimé, son rôle est absorbé par `forge-ui`.

`forge-runtime` est le crate le plus avancé actuellement. `forge-workspace` porte désormais `Workspace` (ADR-0008). `forge-editor` porte `Editor`, premier vrai `Plugin` enregistré par le binaire (ADR-0009). Les crates neuves de la topologie étendue (`forge-plugin-host`, `forge-fs`, `forge-lsp`, `forge-terminal`, `forge-git`) sont encore des squelettes vides, tout comme `forge-ui`, `forge-command`, `forge-cli`.

## Architecture du runtime

Le runtime (`forge-runtime`) suit un modèle de type "runtime applicatif" plutôt qu'un simple `#[tokio::main]` :

```
forge (binaire)
 ├── main.rs           → instancie Runtime, appelle run()
 └── (rien d'autre : pas de logique métier)

forge-runtime
 ├── runtime.rs         → Runtime, orchestrateur central, lifecycle
 ├── state.rs           → RuntimeState (Created/Starting/Running/Stopping/Stopped)
 ├── handle.rs          → RuntimeHandle (API publique minimale pour notifier le runtime)
 ├── task_manager.rs    → TaskManager (gestion centralisée des tâches tokio via JoinSet)
 ├── error.rs           → RuntimeError (thiserror)
 ├── signal.rs           → écoute Ctrl+C / SIGTERM (privé au crate, pub(crate))
 ├── application.rs
 └── lib.rs

forge-event
 └── event.rs           → AppEvent (bus d'événements, ADR-0011)

forge-plugin
 ├── plugin.rs          → trait Plugin (init(&RuntimeContext)), enregistrement statique
 └── context.rs         → RuntimeContext (handle + config + workspace, à donner aux plugins, ADR-0011)
```

`AppEvent` et `Plugin`/`RuntimeContext` ont déménagé hors de `forge-runtime`
vers `forge-event`/`forge-plugin` (ADR-0011) ; `forge-runtime` en dépend au
lieu de les porter directement.

### Principes retenus (voir docs/adr/)

- **Runtime propriétaire** (ADR-0001) : le binaire `forge` ne fait qu'instancier `Runtime` et appeler `run()`. Toute la logique de cycle de vie, tâches, événements vit dans `forge-runtime`.
- **Bus d'événements** (ADR-0002) : communication via un `mpsc::channel<AppEvent>`. Le `Runtime` est le seul consommateur (`event_receiver`). Les producteurs externes utilisent `RuntimeHandle`. `AppEvent` vit dans `forge-event` (ADR-0011).
- **RuntimeHandle** (ADR-0003) : API minimale et `Clone`, donnée aux tâches internes (ex: signal handler) pour qu'elles puissent émettre des événements (ex: `shutdown()`) sans connaître l'état interne du runtime.
- **TaskManager centralisé** (ADR-0004) : toutes les tâches tokio sont enregistrées via `TaskManager::spawn` (méthode `pub(crate)`, pas exposée en dehors du crate). Permet un `shutdown()` groupé avec timeout (5s par défaut), au-delà duquel les tâches restantes sont `abort_all()`.
- **Lifecycle interne au Runtime** (ADR-0005) : les transitions d'état (`RuntimeState`) sont pilotées uniquement par `Runtime::transition_to()`, jamais par le bus d'événements. `AppEvent` ne sert qu'à transporter des intentions externes (ex: `ShutdownRequested`), pas l'état interne.
- **Signal handling dans le runtime** (ADR-0006) : l'écoute de Ctrl+C / SIGTERM est démarrée automatiquement par le `Runtime` lui-même (pas par le binaire). `signal.rs` est dans `forge-runtime`, module privé (`mod signal;`, pas `pub mod`).
- **Chargement de configuration** (ADR-0007) : `Config::load()` fusionne le fichier XDG utilisateur (`dirs::config_dir()/forge/config.toml`) et `./forge.toml` (projet, cwd) — le projet écrase l'utilisateur champ par champ. Fichier absent ou invalide → repli silencieux sur `Config::default()` (un `tracing::warn!` est émis en cas d'erreur de parsing). Jamais d'échec bloquant.
- **Workspace optionnel** (ADR-0008) : `Workspace::open(root)` valide que la racine existe et est un dossier. `Config.workspace_root` absent ou invalide → le runtime démarre avec `workspace: None` (`tracing::warn!` sur erreur), jamais bloquant — même principe que ADR-0007. v1 minimal : `Workspace` ne porte que sa racine, pas de listing de fichiers.
- **Plugins statiques** (ADR-0009) : trait `Plugin` avec un seul hook `init(&mut self, context: &RuntimeContext)`, enregistré via `Runtime::register_plugin(Box<dyn Plugin>)` avant `run()`. Compilés dans le binaire, pas de chargement dynamique. `forge-editor::Editor` est le premier consommateur, enregistré dans `main.rs`. Le trait et le contexte vivent dans `forge-plugin` (ADR-0011).
- **Topologie de crates étendue** (ADR-0011, supersède ADR-0010) : chaque responsabilité transverse (bus d'événements, système de plugin, accès fichiers, LSP, terminal, Git, hébergement de plugins tiers) a sa propre crate plutôt qu'un sous-module interne. Migration en phases — voir `docs/ROADMAP.md` pour l'état d'avancement. Pas de couches `domain/use_cases/infrastructure` imposées à l'intérieur des crates existantes (contrairement à ce que prévoyait ADR-0010, jamais implémenté).
- **`spawn` interne, pas exposé** : `Runtime::spawn` / `TaskManager::spawn` sont `pub(crate)`. Les modules externes ne doivent jamais pouvoir lancer des tâches tokio arbitraires sur le runtime — ils passent par des méthodes dédiées (ex: `register_signal_handler`) ou par `RuntimeContext`.

## Architecture de forge-editor

`forge-editor` reste à plat pour l'instant (`lib.rs` : `Editor`, `Buffer`,
`EditorError`), pas de sous-modules `domain/use_cases/infrastructure` — ADR-0010
prévoyait cette structure mais a été supersédée par ADR-0011 avant d'être
implémentée. `Editor` implémente toujours `Plugin` (import mis à jour vers
`forge-plugin`, ADR-0011) et ne garde qu'un seul `Buffer` actif à la fois.
Sa restructuration réelle (domaine riche, passage sous `forge-ui`) est une
phase ultérieure de la migration ADR-0011, cf. `docs/ROADMAP.md`.

### Flux de shutdown

```
Ctrl+C / SIGTERM
      ↓
signal.rs (tokio::select! sur ctrl_c() et SIGTERM)
      ↓
RuntimeHandle::shutdown()
      ↓
Sender<AppEvent>::try_send(ShutdownRequested)
      ↓
event_loop() (dans Runtime::run, via block_on)
      ↓
handle_event() → RuntimeAction::Stop
      ↓
transition_to(Stopping)
      ↓
TaskManager::shutdown(timeout) — attend les tâches, sinon abort_all()
      ↓
transition_to(Stopped)
```

## Points techniques importants à connaître

- **`JoinSet::spawn` doit être appelé dans un contexte Tokio actif.** Comme `register_signal_handler()` (qui spawn via `TaskManager`) est appelé dans `run()` avant le `block_on`, il faut un guard explicite : `let _guard = self.tokio_runtime.enter();` en début de `run()`, maintenu vivant jusqu'à la fin de la fonction. Alternative envisagée mais non retenue pour l'instant : passer un `tokio::runtime::Handle` au `TaskManager` et utiliser `spawn_on`.
- `event_loop` est une fonction associée (pas une méthode `&mut self`) qui n'emprunte que `event_receiver`, pour éviter les conflits de borrow avec `&self.tokio_runtime` lors du `block_on`.
- `handle_event` est une fonction pure (`AppEvent → RuntimeAction`), testable sans tokio.
- Dépendances tokio : `signal` doit être dans `forge-runtime/Cargo.toml` (pas dans `forge/Cargo.toml`), puisque c'est le runtime qui gère les signaux (voir ADR-0006).

## Prochaines étapes envisagées

Voir `docs/ROADMAP.md` pour les jalons à venir (premier affichage TUI, listing
de fichiers, buffers multiples, etc.).

## Conventions de travail

- Commits en anglais, format conventionnel (`feat:`, `refactor:`, `fix:`...).
- Les ADR sont dans `docs/adr/`, un fichier par décision (`docs/adr/0001-runtime-proprietaire.md`, etc.). Toute nouvelle décision d'architecture significative doit donner lieu à un nouvel ADR.
- Erreurs via `thiserror` dans les crates de librairie (`forge-runtime`), `anyhow` dans le binaire (`forge`).