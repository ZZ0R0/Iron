# V13 — PLANIFICATION (phase 2)

> Découpage d'**exécution** (vagues × pistes). Cadrage = `../*.md`. Nommage `planify-<vague>.<piste>.md`
> (vague = rang séquentiel, piste = track parallèle ; **PAS** un n° de spec — chaque planif **cite** sa spec).
> V13 **réutilise** le pont V12 (sentinelles `@read`/`@arr`/`wraps`, idiomes, ExternApp) — **pas de nouveau
> squelette**. Les 3 capacités touchent toutes `bind.rs`/`lower.rs` → **vagues SÉQUENTIELLES** (ownership non
> disjoint), ordonnées par **levier × tractabilité** (le sûr d'abord, le dur/prototype en dernier).

## Carte des vagues

| Planif | Vague | Réalise | Débloque | Tractabilité |
|---|:--:|---|---|---|
| `planify-1.1` | 1 | [24500] sous-cas A (octets typenum) | **aes-gcm, chacha20** | ✅ sûr (étend `@arr`) |
| `planify-2.1` | 2 | [30200] niveau 1 (`Display`→`.to_string()`) | **chrono (strftime)** | ✅ sûr (nouvel idiome) |
| `planify-3.1` | 3 | [52800]w (io-écriture, `@write`) | **flate2, printpdf** | moyen |
| `planify-4.1` | 4 | [24500] sous-cas B (dimensions matrices) | nalgebra, ndarray, rustfft, rapier2d | ⚠️ **dur, peut rester partiel** |
| `planify-5.1` | 5 | [30200] niveau 2 (zéro-copie tenu) | midly, quick-xml(events) | ⚠️ **prototype** |

> Vagues 1-3 = les **sûrs** (4 crates : aes-gcm, chacha20, chrono-format, flate2/printpdf). Vagues 4-5 = les
> **incertains** (typenum-dimensions, zéro-copie) — on les fait, mais avec **gate-partiel honnête** : ce qui
> aboutit est vert, ce qui résiste est **noté comme limite**, jamais maquillé.

## Ordre & gates
1. `planify-1.1` → aes-gcm chiffre/déchiffre (run réel). 2. `planify-2.1` → chrono `.format(...)` rend une chaîne.
3. `planify-3.1` → flate2 round-trip + printpdf save. 4. `planify-4.1` → nalgebra `Vector3.new().norm()` **si**
la table aboutit. 5. `planify-5.1` → midly/quick-xml **si** clone-owned suffit. **Gate VERT 24/0 + 0 unsafe à
chaque greffe** ; les `.iri` du gate restent commités (filet).

## Plafond visé (honnête)
~27 (V12) + 1.1(2) + 2.1(1) + 3.1(2) = **~32 sûrs** ; +4.1/5.1 si ça aboutit → **jusqu'à ~37-39**. Les ~7
edge-cases (outillage/API, cf. `../README.md §4`) et les ~34 HW **restent hors V13**.

## Invariant méthodo
Chaque vague : build vert + gate vert + 0 unsafe (I2). Réutilise le contrat V12 (pas de modif des sentinelles
existantes). Un écart se **note** (limite documentée), il ne se code pas en douce.
