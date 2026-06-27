# V10.1 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert). 5 pistes · 4
> vagues · refactor du moteur V10 vers l'inférence pilotée par les sites d'appel.

## Avancement des pistes

| Piste | Décisions | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | D-g(struct)·D-i | Fondation : boucle de point fixe + `Promoted` + `gen` réentrant | 1 | ☐ todo |
| 2.1 | **D-a**·D-b·D-g(cœur) | Bascule monomorphe + promotion sur conflit | 2 | ☐ todo |
| 3.1 | **D-g**(incrémental)·D-h·D-d | Incrémental structurel (champ/méthode/index) + ordre supérieur | 3 ∥ | ☐ todo |
| 3.2 | **D-c/R2**·D-j·D-l | Display→générique borné + defaulting final + retours génériques | 3 ∥ | ☐ todo |
| **4.1** | D-k·D-f·gate | **Gate V10.1** — fixtures + verdict + I15 | 4 | ☐ todo |

## Couverture des décisions cadrées (traçabilité — TOUT est planifié)

D-a→2.1 · D-b→2.1 · D-c→3.2 · D-d→3.1 · **D-e→reporté (non codé, cas rare)** · D-f→4.1(doc) · D-g→1.1+2.1+3.1
· D-h→3.1 · D-i→1.1 · D-j→3.2 · D-k→4.1(doc) · D-l→3.2 · R1→1.1/2.1/3.1 · R2→3.2 · R3→subsumé par R1 (2.1).

## Invariant dur de réalisation

À chaque vague la colonne passe l'e2e (corpus 50 + binding 30 + vitrine + zero-annot). **Vague 2 = bascule
atomique** : `id` polymorphe (par promotion) ET `count(xs)` inféré depuis l'appel, validés **ensemble** avant
de clore. Mono seul casserait `id` → ne pas avancer tant que la promotion ne le rattrape pas.

## Écarts / faiblesses (à remplir en réalisation)

- (rien encore)
