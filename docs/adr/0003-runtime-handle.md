# ADR-0003 — RuntimeHandle

## Statut
Accepted

## Contexte
Les tâches doivent interagir avec le runtime sans accéder à son état.

## Décision
Introduire un `RuntimeHandle` exposant uniquement les opérations autorisées (par exemple `shutdown()`).

## Conséquences
API minimale, encapsulation et meilleur contrôle.
