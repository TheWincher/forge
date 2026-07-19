# ADR-0005 — Cycle de vie du runtime

## Statut
Accepted

## Contexte
Le runtime possède plusieurs états : Created, Starting, Running, Stopping, Stopped, Failed.

## Décision
Le runtime pilote lui-même ses transitions via `transition_to(...)`. Les événements expriment uniquement des intentions.

## Conséquences
Machine d'état claire et découplage entre événements et transitions.
