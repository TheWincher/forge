# ADR-0002 — Bus d'événements

## Statut
Accepted

## Contexte
Plusieurs composants (signaux, plugins, API...) doivent pouvoir demander l'arrêt.

## Décision
Utiliser un canal Tokio `mpsc` transportant des `AppEvent`. Le runtime est le seul consommateur.

## Conséquences
Découplage, extensibilité et facilité de test.
