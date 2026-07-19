# ADR-0007 — Chargement de la configuration

## Statut
Accepted

## Contexte
`forge-config` doit charger une configuration réelle depuis le disque, en combinant
des réglages utilisateur et des réglages de projet.

## Décision
- Deux sources sont fusionnées : le fichier utilisateur XDG
  (`dirs::config_dir()/forge/config.toml`) et le fichier projet `./forge.toml`
  (cwd, pas de remontée façon git). Le projet écrase l'utilisateur, champ par champ.
- Le chargement ne peut jamais échouer de manière bloquante : un fichier absent ou
  invalide retombe silencieusement sur `Config::default()`. Une erreur de parsing
  déclenche un `tracing::warn!` (visible dans les logs) mais n'interrompt jamais
  le démarrage.
- `Config::load()` est le seul point d'entrée public ; la logique de fusion/parsing
  est isolée dans des fonctions privées prenant des chemins explicites, pour rester
  testable sans toucher `dirs::config_dir()`.

## Conséquences
### Avantages
- Démarrage robuste (jamais d'échec dû à la config).
- Logique testable indépendamment de la découverte de chemins réels.

### Inconvénients
- Une erreur de configuration silencieuse peut passer inaperçue si les logs ne
  sont pas surveillés.
