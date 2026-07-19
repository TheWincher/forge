# ADR-0009 — Plugins statiques

## Statut
Accepted

## Contexte
`forge-runtime` doit permettre à de futurs modules (éditeur, workspace UI, etc.)
de s'initialiser avec un `RuntimeContext`, sans que `forge-runtime` connaisse
leurs types concrets à l'avance.

## Décision
- Les plugins sont **compilés statiquement** dans le binaire : un trait `Plugin`
  (`fn init(&mut self, context: &RuntimeContext)`), enregistré via
  `Runtime::register_plugin(Box<dyn Plugin>)` avant `run()`. Pas de chargement
  dynamique (`.so`/`.dll`, `libloading`) pour l'instant — prématuré tant qu'il
  n'y a pas de consommateur concret, et ça évite tout problème de stabilité ABI.
- Périmètre v1 volontairement minimal : un seul hook `init`, appelé une fois par
  plugin au démarrage (`Runtime::run()`, après l'entrée dans le contexte Tokio).
  Pas de hook `shutdown` ni d'accès aux événements pour l'instant.
- `Runtime` reste le seul point d'enregistrement (`pub fn register_plugin`) ;
  le binaire `forge` n'enregistre encore aucun plugin réel (rien à enregistrer
  tant qu'aucun module concret n'implémente `Plugin`), donc ADR-0001 (le binaire
  ne fait qu'instancier `Runtime` et appeler `run()`) reste respecté pour l'instant.

## Conséquences
### Avantages
- Aucune complexité FFI/ABI ; le trait `Plugin` est un point d'extension simple
  et sûr pour de futurs crates (`forge-editor`, etc.).
- Périmètre minimal, facile à faire évoluer (hook `shutdown`, accès aux
  événements) une fois qu'un vrai consommateur existera.

### Inconvénients
- Un vrai système de plugins tiers (chargés dynamiquement, hors du binaire)
  nécessitera une révision de cette décision.
