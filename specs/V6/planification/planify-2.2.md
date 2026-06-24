# [2.2] Partage explicite `ref`/`Rc`/`Weak` + warning aliasing (30300)

- Spec parente : 30300 · V6 · **Vague 2** · Statut : todo · Dépend de : 1.1

## But
Honorer **I5** (partage explicite `Rc`/`Weak`) et le **warning I3** (« tu voulais peut-être partager »).

## Fichiers possédés
`src/ir.rs` (usage de `Type::Shared`), `src/lex.rs`+`parse.rs` (`ref`/`weak`), `src/lower.rs`, `src/resolve.rs` (warning).

## Étapes
1. `ref(x)` → `Rc::new(...)` / `ref T` → `Rc<RefCell<T>>` ; accès via `borrow()`/`borrow_mut()`.
2. `weak(r)` → `Rc::downgrade` ; `Weak` documenté (cycles à la charge utilisateur, I5).
3. Warning aliasing mutable auto-cloné (`--audit`, I4).
4. **Zéro `unsafe`** (I2).

## Critère de done
Graphe à ≥2 propriétaires d'un nœud, muté via `ref` → compile + exécute ; `weak` casse un cycle ; warning émis sur le cas piège ; corpus 22/22 inchangé.

## Notes
`Rc<RefCell>` : emprunts dynamiques `borrow()` → panics runtime possibles (documenter).
