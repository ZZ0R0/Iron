# [2.3] Closures / lambdas + `map`/`filter` (20700)

- Spec parente : 20700 · V6 · **Vague 2** · Statut : todo · Dépend de : 1.1

## But
Fonctions anonymes capturant l'environnement → closures Rust ; débloque `map`/`filter`.

## Fichiers possédés
`src/parse.rs` (lambda), `src/infer/` (`Type::Fn`), `src/lower.rs` (closure + `map`/`filter`).

## Étapes
1. Parser la syntaxe lambda (`fn(x): expr`) → `ExprKind::Lambda`.
2. Inférence : `Type::Fn(params, ret)` ; types de params de lambda laissés à rustc (I2/I6).
3. Lowering → closure Rust `|x| …` ; capture par valeur (I1).
4. Built-ins `map(lst, f)` / `filter(lst, f)` → `.iter().map/.filter(..).collect()`.

## Critère de done
Lambda stockée + appelée ; `map`/`filter` sur `list[T]` → compile + exécute ; corpus 22/22 inchangé.

## Notes
Stockage d'une closure → `Box<dyn Fn>` ou générique `impl Fn` — projection à figer (cf. spec 20700 risques).
