# planify-4.1 — Unification : gate corpus + raytracer complet (emit→build→run) + perf + polish

- Réalise (cadrage) : toutes ([30500] · [50700] · [21800]) + polish §2.7
- Version : V16 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1, 2.1, 2.2, 2.3 (greffe de toutes les pistes)

## But
Greffer toutes les pistes, **re-valider l'e2e RÉEL** (pas seulement les tests isolés) et confirmer le critère
de fin : le **bon Rust de raytracer** (`&Scene`/`&Ray`/`&élément`, sans clone) qui compile, tourne et produit
la **bonne image**, **plus vite** que la V15. Plus le polish stdlib (§2.7).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/lower.rs` (polish §2.7 : `int(f64)`→`as i64`, `f64 ** n`→`powi/powf`) — petit.
- `specs/V11-test-2/CARTE-80.md` / un `ETAT` V16 (consignation mesurée).

## Interface produite / consommée
**Consommée** : l'ensemble du moteur post-pistes. **Produite** : un bilan **mesuré** (corpus N/80, perf raytracer).

## Étapes
1. **Corpus 80** : re-jouer la batterie (les programmes qui passaient passent toujours) — zéro régression.
2. **Raytracer e2e** : `ironc emit main.irn` → `cargo build --release --manifest-path main-rust/Cargo.toml` →
   `./main-rust/target/release/iron_out` → `output.png`. Vérifier : (a) build vert, (b) image **identique** à
   l'oracle Python (ou au PNG de référence), (c) **temps mesuré** vs la version V15 (qui clone) — gain attendu
   croissant avec le nombre d'objets.
3. **Polish §2.7** : `int(x)` sur un `f64` → cast (pas le parseur de chaîne) ; `x ** n` sur `f64` → `powi`/`powf`.
4. Consigner : modes whole-program OK, perf raytracer (M-rays/s), corpus N/80, frictions levées.

## Definition of Done (vérifiable)
- **Gate e2e** : corpus 80 vert **ET** raytracer `output.png` correct **ET** plus rapide que V15 (chiffre mesuré).
- `int(3.9)` → `3` ; `2.0 ** 5` → `32.0` (run réel).
- `cargo test -p ironc` vert ; 0 `unsafe` (I2).

## Notes
- C'est le **gate de fin de V16**. Si le raytracer n'est pas « bon » (clone résiduel, image fausse, pas plus
  rapide), on **ré-itère** sur les pistes concernées avant de clore (boucle de vérité, phase 4).
- Image identique : comparer hash/pixels au PNG de référence `raytracer_iron/output.png` (déterminisme du RNG à
  vérifier ; sinon comparer visuellement / quelques pixels-témoins).
