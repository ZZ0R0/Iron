# Iron — V1 / réalisation (trace)

> Phase 3 terminée. **Gate V1 vert (2026-06-24).** V1 = optimisation pure (élision d'emprunt).

## Gate V1 — atteint

| Critère (README §5) | État | Preuve |
|---|---|---|
| Sémantique **inchangée** vs V0 | ✅ | corpus **11/11 sorties identiques** |
| **Moins de `.clone()`** | ✅ | corpus : **4 → 0** clones (A/B vs all-ByValue) |
| `--audit` liste les clone-fallbacks (off par défaut, I4) | ✅ | `a_audit.irn:3:9: perf: clone-fallback` |
| Zéro `unsafe` (I2) ; I1 préservé | ✅ | `forbid(unsafe_code)` ; tout compile |
| Aliasing mutable jamais auto-cloné (I3) | ✅ | N/A V1 (pas de mutation-through-param) |

`cargo test` : 3/3. Exemple concret : `dist2(p: &Pt)` + `dist2(&p)` (V0 : `dist2(p.clone())`).

## Pistes livrées

- **2.1 escape analysis** (`resolve.rs`) : `ParamModes` par fonction — param `ByRef` (`&T`) ssi **toutes**
  ses utilisations sont des emprunts (champ/index/comparaison/`print`/`len`/`for`) et qu'il ne s'échappe pas ;
  sinon `ByValue`. Conservateur (I2).
- **2.2 lowering emprunts** (`lower.rs`) : signatures `&T`, call sites `&x`, `print`/`len` empruntent,
  clone-fallback collecté ; `--audit` dans le driver/CLI.
- **1.1 / 3.1** : contrat étendu (`Disposition::Borrow`, `ParamMode`/`ParamModes`) ; unification = le gate.

## Écarts & dettes → cadrage V2

- **DV1 — Move/Borrow *locaux* non implémentés.** L'`OwnershipMap` reste vide (locales : clone si non-`Copy`).
  Sur le corpus actuel, l'élision **param + `print`** a suffi (0 clone restant). La passe de liveness
  « move à la dernière utilisation » des locales = **dette** (gain sur programmes à locales non-`Copy` réutilisées).
- **DV2 — Escape analysis conservatrice 1-shot.** Un param passé à un `fn` utilisateur est traité comme
  s'échappant (`ByValue`), sans point fixe inter-signatures. Rate l'emprunt transitif. **Dette.**
- **DV3 — `Disposition::Borrow` ajouté mais inutilisé** : les emprunts sont *structurels* (via `ParamModes`
  + `print`), pas une disposition par-lecture. Variante réservée pour DV1.
- **DV4 — Comparaison de structs** : pas de `PartialEq` dérivé (inchangé depuis V0) ; `p == q` sur struct
  échouerait — à cadrer si besoin.

## Phase 4 (boucle de vérité)
Le critère « sémantique inchangée » **est** le test de non-régression (corpus diff = vide). Métrique de
vérité (corpus large) toujours différée.
