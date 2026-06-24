# [3.2] Motifs `match` avancés (20600)

- Spec parente : 20600 · V6 · **Vague 3** · Statut : todo · Dépend de : 1.1

## But
Compléter les motifs : gardes, variantes à champs nommés, imbrication.

## Fichiers possédés
`src/parse.rs` (motifs + variantes nommées), `src/lower.rs` (lower_pattern), `src/infer/` (binding).

## Étapes
1. Garde : `motif if cond:` → `pat if cond => …`.
2. Variantes à champs nommés (`enum E: V(a: i64)`) + motif `E.V(a, b)` / `E.V(a=pa)`.
3. Motifs imbriqués (`Some(Point(x, y))`).

## Critère de done
`match` avec garde + variante nommée + motif imbriqué → compile + exécute ; corpus 22/22 inchangé.

## Notes
Liaison des champs nommés (positionnel vs `champ=motif`) à figer (spec 20600).
