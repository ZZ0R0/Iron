# planify-7.2 — Remplacement non-trivial des projets HW-bound + validation finale du corpus

- Réalise (cadrage) : README V14 §politique de remplacement HW ; valide [50500][50600]
- Version : V15 · Vague : 7 · Piste : 2 · Statut : todo
- Dépend de : planify-6.1 (moteur unifié, on sait ce que C1–C4 visent) — **‖ planify-7.1**

## But
Remplacer les ~34 projets **HW/OS-spécifiques** du corpus par des projets **non-triviaux** (surtout **pas** des
cas jouets) qui **exercent** les classes dures du moteur (C1 dimensions, C2 trait-méthodes, C3 const-gen, C4
flux), puis **mesurer** l'état réel des 80 et le consigner. C'est la **boucle de vérité** (phase 4) : un chiffre
honnête, pas une estimation.

## Fichiers POSSÉDÉS (ownership disjoint)
- `specs/V11-test-2/<nouveaux-projets>/` (les `.irn` + manifestes des remplaçants).
- `specs/V11-test-2/CARTE-80.md`, `specs/V13/ETAT-IMPLEMENTATION.md` (ou un `ETAT` V15) — la consignation.
- **Aucun** `ironc/src/*` ni `ironc/bindings/*.iri` (ceux-ci sont à 7.1) → disjoint.

## Interface produite / consommée
**Consommée** : le moteur++ unifié (vague 6) + les `.iri` curés (7.1) au fur et à mesure.
**Produite** : un corpus de **80 projets significatifs** ; chaque remplaçant respecte les 3 critères (zéro HW ;
exerce ≥1 classe dure ; sortie vérifiable). Un tableau d'état **mesuré** : N/80 qui tournent réellement.

## Étapes
1. Lister les ~34 HW-bound (depuis CARTE-80) ; pour chacun, choisir un **remplaçant non-trivial** dont le crate
   est visé par C1–C4 (ex. un vrai calcul nalgebra, un round-trip AEAD réel, une compression flate2, etc.).
2. Écrire chaque projet remplaçant (`.irn` + `ironc run` qui produit la **bonne sortie**).
3. Lancer l'e2e sur **tout** le corpus ; consigner pour chacun : vert / sauté+raison (depuis le rapport) /
   curé (7.1).
4. Recalculer le **N/80 réel** ; mettre à jour CARTE-80 + l'ETAT V15. **Aucun chiffre estimé** — que du mesuré.

## Definition of Done (vérifiable)
- Le corpus compte **80 projets** ; chaque remplaçant compile + run + produit la bonne sortie.
- Chaque remplaçant **exerce explicitement** ≥1 classe dure (tracé dans CARTE-80).
- Un tableau **mesuré** N/80 « tourne réellement » est consigné ; écart à la cible 60/80 **expliqué par les
  raisons de skip restantes** (honnêteté brutale, pas de verdict gonflé).
- Gate VERT global ; 0 `unsafe`.

## Notes
- **Anti-trivialité (exigence utilisateur dure)** : un remplaçant `hello-world` ou `2+2` est **refusé** — il
  doit mettre le moteur à l'épreuve. La revue de trivialité fait partie du DoD.
- Parallèle strict avec 7.1 (projets vs bindings — types de fichiers distincts).
- Si la cible 60/80 n'est pas atteinte, le **rapport de couverture** dit **exactement** pourquoi et où : c'est
  le livrable de vérité, pas l'échec.
