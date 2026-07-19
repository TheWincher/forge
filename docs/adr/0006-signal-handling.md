# ADR-0006 — Gestion des signaux

## Statut
Accepted

## Contexte
Forge doit gérer Ctrl+C et SIGTERM de manière uniforme.

## Décision
Le runtime démarre automatiquement une tâche d'écoute des signaux qui envoie `AppEvent::ShutdownRequested`.

## Conséquences
Comportement homogène, compatible Docker/Kubernetes et sans code spécifique dans les binaires.
