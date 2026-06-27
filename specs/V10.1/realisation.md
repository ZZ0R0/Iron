# V10.1 — Réalisation (tracker)

> **État : RÉALISÉ — GATE V10.1 VERT (20/20).** Le moteur infère désormais les params **par les sites
> d'appel** (programme entier), polymorphie sur conflit (coût runtime nul), résolution structurelle
> incrémentale, HOF, génériques bornés. 5 pistes · 4 vagues.

## Avancement des pistes

| Piste | Décisions | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | D-g(struct)·D-i | Fondation : boucle de point fixe + `Promoted` + `gen` réentrant | 1 | ✅ fait |
| 2.1 | **D-a**·D-b·D-g(cœur) | Bascule monomorphe + promotion sur conflit | 2 | ✅ fait |
| 3.1 | **D-g**(incrémental)·D-h·D-d | Incrémental structurel (champ/méthode/index) + ordre supérieur | 3 | ✅ fait |
| 3.2 | **D-c/R2**·D-j·D-l | Display→générique borné + defaulting final + retours génériques | 3 | ✅ fait |
| **4.1** | D-k·D-f·gate | **Gate V10.1** — fixtures + verdict + I15 | 4 | ✅ **VERT 20/20** |

## Couverture des décisions cadrées (traçabilité — TOUT est planifié)

D-a→2.1 · D-b→2.1 · D-c→3.2 · D-d→3.1 · **D-e→reporté (non codé, cas rare)** · D-f→4.1(doc) · D-g→1.1+2.1+3.1
· D-h→3.1 · D-i→1.1 · D-j→3.2 · D-k→4.1(doc) · D-l→3.2 · R1→1.1/2.1/3.1 · R2→3.2 · R3→subsumé par R1 (2.1).

## Invariant dur de réalisation

À chaque vague la colonne passe l'e2e (corpus 50 + binding 30 + vitrine + zero-annot). **Vague 2 = bascule
atomique** : `id` polymorphe (par promotion) ET `count(xs)` inféré depuis l'appel, validés **ensemble** avant
de clore. Mono seul casserait `id` → ne pas avancer tant que la promotion ne le rattrape pas.

## Écarts / faiblesses (relevés)

- **D-e reporté (non codé)** : l'anti-unification structurelle (`count` appelé en `list[i64]` **ET**
  `list[str]` → `list[T]`) n'est pas implémentée — promotion « param entier » seulement. Cas rare ; à
  rouvrir sur besoin réel. Sans elle, un tel double-appel tombe sur le filet `rustc`.
- **Bornes de générique promu** : seules `Display`/`PartialOrd`/`PartialEq` sont portées sur un générique
  implicite. Un param promu **arithmétique** (`+`/`-`…) n'a pas de borne propre (un `T + 1` générique est trop
  fin) → `rustc` filet (D-f). En pratique un param arithmétique appelé à deux types est déjà contradictoire.
- **Externes génériques** (`.iri`, `Vec::push<T>`) : hors-scope (D-k), traités opaques. Conforme au « 0.1 % »
  binding.
- **`local.rs`/`flow.rs` conservés** (hérité de V10) : le moteur possède l'inférence d'**annotations**
  (params/retours) ; le typage **expression-par-expression** et le flux Option/Result restent la base saine
  en aval. Les retirer = produire la `TypeMap` complète par nœud (itération à part).
