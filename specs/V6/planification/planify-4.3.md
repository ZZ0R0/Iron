# [4.3] Stdlib élargie (80200)

- Spec parente : 80200 · V6 · **Vague 4** · Statut : todo · Dépend de : 1.1

## But
Compléter le prélude : `sum`/`concat` génériques, tri, IO (stdin), opérations de chaînes.

## Fichiers possédés
`src/infer/local.rs` (types de retour), `src/lower.rs` (built-ins).

## Étapes
1. `sum(lst)` générique (`T: Add`) ; `concat` sur listes ; `sort(lst)` (`T: Ord`).
2. `input()` → lecture stdin → `str`.
3. `split`/`upper`/`lower`/`index_of` sur chaînes.

## Critère de done
Programme `sum` générique + `sort` + `input` + `split`/`upper` → compile + exécute ; corpus 22/22 inchangé.

## Notes
Built-ins = noms réservés (DZ1) — documenter.
