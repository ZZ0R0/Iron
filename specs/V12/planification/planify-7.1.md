# planify-7.1 — Constantes publiques d'un crate (`pub const`) — orthogonale

- Réalise (cadrage) : [50500]
- Version : V12 · Vague : 7 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (pour éviter de collisionner avec la fondation sur bind/iri/lower)

## But
Référencer une **constante** exposée par un crate quand c'est le point d'entrée de l'API (`base64` via
`BASE64_STANDARD.encode(...)`). Gap mineur, orthogonal au pont (mais touche bind/iri/lower → après la fondation).

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bind.rs` (collecter `inner.constant` publics), `ironc/src/iri.rs` (parser
  `extern const NOM: T = "crate::path::NOM"`), `ironc/src/infer/*` (typer `NOM` en `Extern(T)`),
  `ironc/src/lower.rs` (émettre le chemin Rust complet).
- Séquentielle (vague 7).

## Interface produite / consommée
- **Consomme** : — (orthogonal au type bridge).
- **Produit** : nouvelle entrée `.iri` `extern const` ; usage `NOM.method(...)`.

## Étapes
1. `bind.rs` : ramasser les constantes publiques → ligne `.iri` `extern const`.
2. `iri.rs` : parser `extern const NOM: T = "path"`.
3. infer/lower : `NOM` (Var) → `Extern(T)` ; émettre `crate::path::NOM`.

## Definition of Done (vérifiable)
1. `BASE64_STANDARD.encode("hello")` → base64 (programme complet vert).
2. **Gate VERT** ; pas de collision nom de const ↔ var locale (cf. questions ouvertes `50500.md`).

## Notes
- `static mut` / consts génériques = hors périmètre (cf. `50500.md`).
