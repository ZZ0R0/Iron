# planify-4.1 — Bornes de ctor convertibles (Into<X>/TryFrom) — dérivée du pont

- Réalise (cadrage) : [24100b] (sous-cas de [24100])
- Version : V12 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (fondation close)

## But
Reconnaître les constructeurs bornés par une **conversion depuis chaîne/valeur** (`T: Into<X>`, `X: TryFrom<T>`,
newtype-string de crate) → débloque `redis`/`tungstenite`/`ureq`/`clap` (construction). Une **entrée de table**
sur le pont, plus du cas-par-cas.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (étendre la table `BOUND_TO_IRON` : motifs `Into<X>`/`TryFrom<T>` → type
  source mappable), `ironc/src/bind.rs` (consommer l'entrée pour ne plus dropper le ctor).
- Séquentielle (vague 4) : pas de piste concurrente sur ces fichiers.

## Interface produite / consommée
- **Consomme** : la fondation [24000] (tables-données, suivi des params).
- **Produit** : ctors convertibles bindés (`Command.new("x")` où `x: impl Into<clap::Str>`).

## Étapes
1. Table : `Into<X>`/`TryFrom<T>` où `X`/`T` se réduit à `str`/numérique → exposer le type source Iron.
2. `bind.rs` : si la borne du param matche la table, projeter le param au type source (pas de drop).
3. Vérifier que le lowering existant (coercition `str`→`&str`/`.into()`) couvre l'appel (sinon, NOTE pour 2.2-bis).

## Definition of Done (vérifiable)
1. `clap`/`redis`/`ureq` régénérés : le ctor-clé **apparaît** et un appel trivial **compile**.
2. **Gate VERT** ; aucune entrée hors `bridge` pour cette reconnaissance.

## Notes
- Si la conversion exige une closure ou un trait-objet → **hors périmètre** (relève de [40300]), à noter, pas à forcer.
