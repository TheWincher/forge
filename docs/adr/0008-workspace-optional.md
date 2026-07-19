# ADR-0008 — Workspace optionnel

## Statut
Accepted

## Contexte
`forge-workspace` était un squelette vide. `Config.workspace_root` (ADR-0007) pointe
naturellement vers ce crate, mais rien ne garantit qu'un chemin de workspace soit
configuré, ni qu'il soit valide.

## Décision
- Un workspace ouvert est **optionnel** : `forge` doit pouvoir démarrer sans, comme une
  fenêtre vide. Cohérent avec le principe déjà posé en ADR-0007 (aucun échec bloquant
  au démarrage à cause d'une donnée d'environnement externe : fichier de config, ici
  chemin de workspace).
- `Workspace::open` valide que la racine existe et est un dossier ; en cas d'échec
  (chemin absent, invalide, `Config.workspace_root` à `None`), le runtime démarre avec
  `workspace: None` et logue un `tracing::warn!` — jamais d'erreur fatale.
- Premier jet volontairement minimal : `Workspace` porte uniquement sa racine
  (`root: PathBuf`). Pas de listing de fichiers ni d'arborescence pour l'instant —
  base sur laquelle itérer une fois qu'un consommateur concret (éditeur, arbre de
  fichiers UI) en aura besoin.

## Conséquences
### Avantages
- Comportement cohérent avec ADR-0007 : toute donnée d'environnement externe est
  best-effort, jamais bloquante.
- Surface d'API minimale, facile à faire évoluer sans casser de contrat existant.

### Inconvénients
- `Option<Workspace>` doit être propagé partout où le workspace est consommé
  (`RuntimeContext`, futurs modules) — un peu de bruit `Option` à gérer par les
  appelants.
