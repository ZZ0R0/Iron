# [4.2] Modules multi-fichiers Iron (70200)

- Spec parente : 70200 · V6 · **Vague 4** · Statut : todo · Dépend de : 1.1

## But
Plusieurs `.irn` → un programme via `import` (distinct de `use "x.iri"`).

## Fichiers possédés
`src/parse.rs` (`import`), `src/driver.rs` (graphe d'imports, fusion d'`Ast`, multi-`FileId`).

## Étapes
1. Parser `import foo` → `Ast.modules`.
2. Driver : charger le graphe (relatif), fusionner les `Ast`, lowering unifié (mono `main.rs`).
3. Détection de **cycles** d'import ; multi-`FileId` pour la remontée (I8).

## Critère de done
Programme sur 2 fichiers → compile + exécute ; cycle d'import → erreur claire ; corpus 22/22 inchangé.

## Notes
Namespacing (`module.item` vs à plat) à figer (spec 70200).
