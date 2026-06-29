# planify-2.1 — Retour `Display`-borné → `.to_string()` (idiome `display`) → chrono strftime

- Réalise (cadrage) : [30200] niveau 1
- Version : V13 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : —

## But
Un retour externe **emprunté/non-clonable mais `Display`** (`DelayedFormat<'a>` de chrono `.format(...)`) →
émettre `(call).to_string()` (chaîne possédée, plus de lien de durée de vie). Débloque **chrono strftime**.

## Fichiers POSSÉDÉS
- `ironc/src/bind.rs` : `project` — détecter un retour dont le type externe implémente **`Display`** mais n'est
  ni clonable (`clone`) ni `&T`, et n'est pas un type Iron mappable → idiome **`display`**, type Iron `str`.
- `ironc/src/iri.rs` : `parse_idioms` — accepter `display`.
- `ironc/src/ir.rs` : `Idiom::Display`.
- `ironc/src/lower.rs` : idiome `Display` → `(call).to_string()`.

## Interface produite / consommée
- **Produit** : `.iri` où `format(self, fmt: str) -> str = "format" [display]` ; lower émet `.to_string()`.
- **Consomme** : mécanique d'idiomes existante (`clone`/`builder`/`collect`).

## Étapes
1. `ir.rs` : ajouter `Idiom::Display`.
2. `bind.rs project` : si le retour est un type externe `Display`-only (détecter via les impls/bornes rustdoc :
   implémente `Display`, pas `Clone`, pas `&T`) → idiome `display`, ret `str`.
3. `iri.rs` : `parse_idioms` reconnaît `display` → `Idiom::Display`.
4. `lower.rs` : pour `Idiom::Display`, envelopper l'appel en `(call).to_string()`.
5. Régénérer chrono, tester `print(date.format("%A %d %B %Y"))`.

## Definition of Done (vérifiable)
1. chrono : `print(NaiveDate.from_ymd_opt(2026,6,29).unwrap().format("%Y-%m-%d"))` affiche `2026-06-29` (run réel).
2. Gate VERT 24/0 ; 0 `unsafe`.

## Notes
- Détecter « `Display` mais pas clonable » robustement : si trop fragile, restreindre aux retours dont le nom de
  type contient `Format`/`Display` (heuristique bornée) — à trancher à l'implémentation, sans faux positifs.
