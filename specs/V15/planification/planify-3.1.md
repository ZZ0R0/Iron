# planify-3.1 — Rapport de génération & système deux couches (gen/curé) — clôt le squelette

- Réalise (cadrage) : [50400] (deux couches), [50500] (rapport, couverture, --strict)
- Version : V15 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1 (binder à portée connue retournant Coverage + Vec<SkipDiagnostic>)

## But
Rendre la portée **visible à l'utilisateur** (le rapport) et faire **coexister** auto-généré et curé sans
perte. C'est l'étape qui **clôt le squelette e2e** : après elle, `ironc add` informe honnêtement, la
couverture est un nombre, et un binding manuel écrase proprement l'auto.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/driver.rs` (rendu rapport + orchestration gen/curé + `--strict`).
- `ironc/src/iri.rs` (chargement **two-layer** : `<crate>.gen.iri` puis `<crate>.iri`, last-wins par entrée).
- `ironc/src/main.rs` (flags CLI : `--strict`, sortie du rapport).

## Interface produite / consommée
**Consommée** : `(iri, Coverage, Vec<SkipDiagnostic>)` de `bind::emit` (2.1) ; le parseur `.iri` last-wins
existant ([52700]).
**Produite** :
- `<crate>.gen.iri` avec en-tête `# AUTO-GÉNÉRÉ — ne pas éditer ; voir <crate>.iri`.
- `<crate>.iri.report.md` (table : item · kind · reason · hint) + **résumé stderr** + **couverture %**.
- Chargement à l'import : `gen` puis `curé`, **clé de merge `(kind, nom-Iron)`**, curé gagnant ; bloc-type =
  unité atomique d'override.
- Code de sortie `0` malgré des skips ; `--strict` → échec si couverture < seuil.

## Étapes
1. `driver.rs` : sérialiser `Vec<SkipDiagnostic>` en `<crate>.iri.report.md` (lisible) + résumé stderr
   `binding <crate> : N items — B liés, S sautés (couverture P%)` + liste des sautés avec `hint`.
2. Écrire la couche auto dans `<crate>.gen.iri` (en-tête AUTO).
3. `iri.rs` : charger `<crate>.gen.iri` PUIS `<crate>.iri` s'il existe ; le merge last-wins par `(kind, nom)`
   fait gagner le curé. Détecter une **curation orpheline** → `SkipReason::OrphanCuration` (remontée au rapport).
4. `main.rs` : `--strict` (échec si couverture < seuil, défaut à fixer) ; brancher la sortie du rapport.
5. Émettre/valider l'e2e (cf. DoD).

## Definition of Done (vérifiable)
- `ironc add <crate>` écrit `<crate>.gen.iri` (en-tête AUTO) **et** `<crate>.iri.report.md` **et** un résumé stderr.
- **Test override** : un `<crate>.iri` curé typé écrase une entrée `gen` opaque → un programme témoin utilise
  la version curée et compile+run.
- **Test régénération sans perte** : relancer `ironc add` réécrit `.gen.iri`, **hash du `.iri` curé inchangé**.
- **Test zéro-silencieux bout-en-bout** : sur aes-gcm, le rapport liste `encrypt`/`decrypt` en
  `GenericTraitMethod` **avant** toute compilation utilisateur.
- `--strict` échoue sous le seuil, réussit au-dessus. Gate VERT ; 0 `unsafe`.

## Notes
- **C'est le gate de fin de fondation** : à partir d'ici la colonne **passe l'e2e** en permanence (vague 4+
  ne doit jamais le casser).
- ❓ Emplacement définitif des fichiers (cache vs dossier projet) : choisir le plus simple qui marche ;
  documenter. ❓ Seuil `--strict` par défaut : proposer 90 %, ajustable.
