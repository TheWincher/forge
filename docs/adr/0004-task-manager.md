# ADR-0004 — Gestion centralisée des tâches

## Statut
Accepted

## Contexte
Les tâches asynchrones doivent être supervisées et arrêtées proprement.

## Décision
Créer un `TaskManager` responsable du lancement, du suivi et de l'arrêt des tâches.

## Conséquences
Point central pour la supervision, les timeouts et les évolutions futures.
