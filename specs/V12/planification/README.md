# V12 — PLANIFICATION (phase 2)

> **Découpage d'EXÉCUTION** de la V12 (vagues × pistes). Le **cadrage** (le *quoi/contrats*) vit dans
> `../*.md` (`24000.md`, `24100.md`, `24200.md`, `82000.md`, `50500.md`…). Ici = le *dans quel ordre, qui
> possède quoi, en parallèle de qui*. **Nommage : `planify-<vague>.<piste>.md`** — la vague = rang séquentiel,
> la piste = track parallèle (ownership disjoint). Le numéro **n'est PAS** celui d'une spec ; chaque planif
> **cite** la/les spec(s) qu'elle réalise. (Cadrage ≠ planification.)

## Périmètre de la V12 (tranche cohérente)
Le **sous-système de types** : la **fondation-refactor** `[24000]` (le pont de correspondance Iron↔Rust) +
les **génériques** `[24100]` qu'elle réalise + les **dérivées** qui tombent de la même machinerie
([24100b], [82000], [24200], [50500]). **Hors V12** (versions futures, autres fondations) : [30200] retours
empruntés, [52800] io-flux, [40300] closures→dyn, [40400] derive, [52900] serde typé, [24300] impl-extern,
et les **à-prototyper** [35000] event-loop / [53000] async / [43000] no_std (cf. `../README.md` §priorisation).

## Pourquoi une fondation d'abord (rappel du constat)
La frontière de types est encodée **3×** (projection `bind.rs` / inférence `infer/` / coercition `lower.rs`) ;
les crate-bugs B5→B33 sont des cas-par-cas dispersés dans ces 3 endroits. La V12 **tarit la source** : un pont
= source unique + suivi des paramètres de type. Détail : `../24000.md`.

## Carte des vagues × pistes

| Planif | Vague | Piste | Réalise | Rôle | Possède (fichiers) |
|---|:--:|:--:|---|---|---|
| `planify-1.1` | 1 | 1 | [24000] | **Squelette** : module `bridge` + `Type::ExternApp` + `.iri` `Name[args]` + `params` (contrat **gelé**, pass-through vert) | bridge.rs(création), ir.rs, lower.rs(ExternApp), iri.rs, bind.rs(émission si params) |
| `planify-2.1` | 2 | 1 | [24000],[24100] | **Projection** : binder produit les args génériques + tables pilotées-donnée | **bind.rs** (exclusif) |
| `planify-2.2` | 2 | 2 | [24000],[24100] | **Coercition** : numérique unifié + monomorphisation `path::<args>` | **lower.rs** (exclusif) |
| `planify-2.3` | 2 | 3 | [24000],[24100] | **Inférence** : substitution des params (`Point[f64]`.m→`T`=`f64`) | **infer/** (exclusif) |
| `planify-3.1` | 3 | 1 | [24000],[24100] | **Unification** : greffe + e2e RÉEL (geo/nalgebra construisent & calculent) → **Gate [24100]** | jonctions + tests |
| `planify-4.1` | 4 | 1 | [24100b] | Dérivée : bornes de ctor convertibles (`Into<X>`/`TryFrom`) | bridge.rs, bind.rs |
| `planify-5.1` | 5 | 1 | [82000] | Dérivée : ergonomie octets (`bytes`, hex) | bridge.rs, lower.rs |
| `planify-6.1` | 6 | 1 | [24200] | Dérivée : tableaux taille fixe (`[u8;N]`/`GenericArray`) | ir.rs, bridge.rs, lower.rs, bind.rs |
| `planify-7.1` | 7 | 1 | [50500] | Orthogonale : constantes publiques (`pub const`) | bind.rs, iri.rs, infer/, lower.rs |

> **Vagues 1-3 = la fondation + [24100]** (le cœur, fan-out réel sur 3 fichiers disjoints en vague 2).
> **Vagues 4-7 = dérivées**, **séquentielles** entre elles : elles partagent `bind.rs`/`bridge.rs`, donc **pas**
> parallélisables (la règle d'ownership disjoint l'interdit) → une vague chacune, par rentabilité. Elles se
> **re-décomposent finement** au début de leur cycle, une fois la forme finale du pont connue (vague 3 close).

## Ordre & gates
1. `planify-1.1` → **Gate de fondation** (traversée `ExternApp` e2e, pass-through vert).
2. `planify-2.1‖2.2‖2.3` (parallèle, ownership disjoint) → chacune verte isolément.
3. `planify-3.1` → **Gate [24100]** (geo/nalgebra compilent **et tournent**).
4. `planify-4.1` → `5.1` → `6.1` → `7.1`, gate vert à chaque greffe.

## Invariant méthodo (dur)
La colonne **passe l'e2e à chaque étape** : squelette pass-through d'abord, fan-out par fichier, unification,
e2e réel — jamais de big-bang. `.iri` du gate commités ⇒ le gate reste le filet pendant tout le refactor.

## Test mental de non-collision (validé)
Vague 2 : `2.1`=bind.rs, `2.2`=lower.rs, `2.3`=infer/ → **disjoints**, et tous contre le **schéma gelé** par
`1.1` (aucune interface non figée attendue). ✅ Parallélisables. Vagues 4-7 : partagent bind/bridge → **séquentielles**. ✅
