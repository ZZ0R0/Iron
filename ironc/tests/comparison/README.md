# Comparaison-test utilisateur (V11, gate 90900)

Étape **finale** de V11 : le test prévu par l'utilisateur, comparé à une référence.

## Comment l'ajouter

Déposer ici **deux fichiers de même nom** :

- `mon_test.irn`      — le programme Iron à compiler + exécuter
- `mon_test.expected` — la sortie attendue (stdout exact)

`tests/gate-v11.sh` (bloc [6]) compile, exécute et **compare** automatiquement chaque `.irn`
à son `.expected`. Plusieurs paires sont permises.

Tant qu'aucune paire n'est présente, le gate marque le bloc ⏸️ (en attente), sans échouer.
