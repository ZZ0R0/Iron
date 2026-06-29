# planify-2.2 — Résolution d'import `.iri` récursive (tous les modules)

- Réalise (cadrage) : [50700]
- Version : V16 · Vague : 2 · Piste : 2 · Statut : todo
- Dépend de : planify-1.1 figée (démarrable en parallèle de 2.1/2.3 — fichier disjoint)

## But
Permettre à n'importe quel **sous-module** Iron d'importer un **crate** (binding `.iri`) là où il l'utilise.
Aujourd'hui un `import mathx` dans un sous-module échoue (`No such file: mathx.irn`) : la résolution
`module .irn` **ou** `binding .iri` n'existe qu'au fichier d'entrée.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/driver.rs` **uniquement** (la résolution d'import). Disjoint de resolve/lower/infer → **parallèle**.

## Interface produite / consommée
**Consommée** : la routine de résolution existante de `compile()` (curé → `.irn` → `.gen.iri`/`.iri`).
**Produite** : la **même** résolution appliquée récursivement dans `load_module`.

## Étapes
1. **Factoriser** la résolution d'un import en une routine unique : registre curé (`import std`) → module `X.irn`
   → binding `X.gen.iri`/`X.iri` (cf. `compile()` `driver.rs:62-93`).
2. Dans `load_module` (`driver.rs:~175`), remplacer `for sub in &mod_ast.modules { load_module(sub, …) }` (qui
   suppose `.irn`) par l'appel à cette routine — un sous-module peut donc résoudre un binding.
3. Les bindings d'un sous-module alimentent le `Bindings` global (fusion last-wins existante). Erreur **claire**
   si ni module ni binding (même message qu'à la racine).
4. Réutiliser les gardes `loaded`/`loading` (cycles/diamants).

## Definition of Done (vérifiable)
- Test multi-fichiers : un sous-module fait `import <crate>` et utilise une fn du crate → compile + run.
- Le raytracer compile avec `import mathx`/`fastrand` **déplacés** dans le module qui les utilise (plus besoin
  de tout remonter dans `main.irn`).
- Corpus 80 vert.

## Notes
- Petite piste, contenue à `driver.rs`. Aucun risque sur l'ownership/codegen.
