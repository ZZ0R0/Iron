# [1.1] Fondation V6 : gel des nouveaux contrats IR

- Spec(s) : transverse · V6 · **Vague 1** (séquentielle) · Statut : todo · Dépend de : —

## But
Étendre `ir.rs` avec **tous** les nouveaux contrats partagés V6, en additif/stubs, **corpus 22/22 vert**
(aucune feature activée). Gèle le schéma pour le fan-out.

## Fichiers possédés
`src/ir.rs` ; arms défaut dans les matches exhaustifs (`lower_type`, `is_copy`, `infer`, `resolve`, `lower`).

## Interface produite (gelée)
- `Type::Shared(Box<Type>)` (Rc, 30300) · `Type::Fn(Vec<Type>, Box<Type>)` (closures, 20700).
- `ExprKind::Lambda { params, body }` (20700).
- `Pattern` : champ `guard: Option<Expr>` + sous-motifs nommés (20600).
- `ItemKind::Enum` : variantes à **champs nommés** (20600).
- `Ast.modules: Vec<String>` (imports Iron, 70200).

## Critère de done
`cargo build` vert ; **corpus 22/22 inchangé** ; `ir.rs` expose les nouveaux contrats ; arms défaut partout
(pas de `match` non exhaustif).

## Notes
Gèle `ir.rs` : les vagues 2+ remplissent leur logique sans re-toucher les contrats.
