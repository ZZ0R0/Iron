# planify-3.1 — Unification + validation [24100] (greffe des 3 pistes)

- Réalise (cadrage) : [24000], [24100]
- Version : V12 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1, planify-2.2, planify-2.3

## But
Greffer les trois pistes de la vague 2 sur le squelette et **revalider l'e2e RÉEL** (pas seulement les tests
unitaires de chaque piste). Clôt la fondation [24000] et **valide [24100]** sur de vrais crates.

## Fichiers POSSÉDÉS (ownership disjoint)
- Séquentiel (post-fan-out) : peut toucher les points de **jonction** entre `bind`/`lower`/`infer` (routage
  `ExternApp` finalisé côté `lower` avec les types instanciés de `infer`). Pas de piste parallèle concurrente.
- Édite au besoin : `ironc/src/lower.rs` (finalisation routage), fixtures de gate, `ironc/tests/`.

## Interface produite / consommée
- **Consomme** : `.iri` à params (2.1), monomorphisation (2.2), substitution (2.3).
- **Produit** : la colonne [24100] intégrée qui **compile et tourne** sur les cibles de vérité.

## Étapes
1. Brancher les types instanciés de 2.3 dans le routage `ExternApp` de 2.2 (retours `T` concrets).
2. Régénérer les `.iri` cibles et faire **compiler + tourner** des programmes complets :
   - `geo` : `Point.new(2.0,3.0)` → `.x()/.y()`, `euclidean_distance` rendent `f64`.
   - `nalgebra` : `Vector3.new(1.0,2.0,3.0)` → `.norm()` `f64`.
   - `ndarray`/`polars`/`rustfft` : ctor scalaire-générique construit (au moins un calcule).
   - `rusqlite` lecture : si `Row::get::<T>` se débloque via génériques, le valider ; **sinon noter** que la
     closure `query_row(|row| …)` relève de [40300] (cadré ailleurs) — **ne pas forcer** ici.
3. Geler la **fixture de validation [24100]** (geo/nalgebra) dans le gate.

## Definition of Done (vérifiable)
1. `geo`/`nalgebra` : programme complet **construit + calcule** (sortie numérique correcte), `cargo` vert.
2. **Gate VERT 24/0** + fixtures [24100] vertes = **Gate [24100]**.
3. 0 `unsafe`, pipeline I9 intact. → la fondation est close ; la vague 4 (dérivées) peut démarrer.

## Notes
- Invariant dur méthodo : la colonne **passe l'e2e** à la sortie de cette vague — sinon ré-unifier avant d'ouvrir la vague 4.
- Tout blocage résiduel d'un crate cible qui relève d'une AUTRE capacité (closures/io/derive) se **note** pour son cycle, ne se code pas ici.
