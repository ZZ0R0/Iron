# planify-8.2 — serde **typé** = param `Deserialize` (RACINE 1) + derive émis (RACINE 2)

- Réalise (cadrage) : [52900]
- Version : V12 · Vague : 8 · Piste : 2 · Statut : todo
- Dépend de : planify-8.1 (derive émis), planify-3.1 (substitution génériques)

## But
Faire marcher `serde_json::from_str::<MonStruct>(s)` **typé** (et toml/yaml/bincode doc) : la **conjonction** de
deux racines — le param générique `T: Deserialize` (stratégie RACINE 1) + le `#[derive(Deserialize)]` émis sur
`MonStruct` (RACINE 2). Démontre la **cohérence** de l'architecture (deux racines qui se composent).

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (entrée `STRATEGIES` : borne `Deserialize`/`Serialize` → struct Iron
  éligible), `ironc/src/lower.rs` (émettre `from_str::<MonStruct>`), `ironc/src/infer/` (typer le retour `T`).
- Séquentielle (vague 8, après 8.1).

## Interface produite / consommée
- **Consomme** : derive émis (8.1) + substitution (2.3) + pont (1.1).
- **Produit** : (de)sérialisation **typée** vers/depuis un struct Iron.

## Étapes
1. `STRATEGIES` : `T: Deserialize` → `accepts = ` un type-struct Iron dérivé ; `T: Serialize` idem en entrée.
2. lower : émettre `serde_json::from_str::<#MonStruct>(&s)` et `to_string(&v)`.
3. Vérifier la composition derive(8.1)×param(ici) sur un round-trip complet.

## Definition of Done (vérifiable)
1. `cfg = from_str::<Config>(toml_str)` peuple un struct Iron, champs lisibles (run réel).
2. round-trip `to_string` → `from_str` égal. **Gate VERT** ; 0 `unsafe`.

## Notes
- C'est la preuve que les racines **composent** : aucune des deux seule ne suffit ; ensemble, serde typé marche.
