# ADR-0001 — Runtime propriétaire

## Statut
Accepted

## Contexte
Forge nécessite un runtime responsable du cycle de vie, des tâches asynchrones, des événements internes et d'un arrêt propre.

## Décision
Créer une structure `Runtime` qui encapsule `tokio::runtime::Runtime`. Le binaire ne fait qu'instancier le runtime puis appeler `runtime.run()`.

## Conséquences
### Avantages
- séparation des responsabilités
- contrôle du cycle de vie
- extensible

### Inconvénients
- un peu plus de code
