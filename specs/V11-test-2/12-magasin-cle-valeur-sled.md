# 12 — Magasin clé-valeur embarqué persistant
> **Domaine** Bases de données · **Crate(s)** `ironc add sled` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un store clé→valeur persistant sur disque (type embedded KV). En entrée : des paires `clé/valeur` et des opérations `put/get/del/scan` ; en sortie : la valeur lue, ou les paires d'un préfixe donné. Les données survivent au redémarrage (B-tree sur fichier).

## Ce que ça teste dans Iron
Une API « feuille » mais qui expose un **type maison** (`IVec`, tampon d'octets à conversion implicite) et des **itérateurs de `Result<(IVec, IVec)>`**. Stresse la conversion octets↔texte aux frontières et le bound générique `AsRef<[u8]>` sur clés/valeurs.

## Approche
- `sled::open(path)` → `Db`.
- `db.insert(key, value)` / `db.get(key)` → `Option<IVec>` / `db.remove(key)`.
- Conversion : `IVec` → `&[u8]` → `String` (UTF-8) côté Iron.
- Itération : `db.scan_prefix(pref)` → itérateur de `Result<(IVec, IVec)>` ; `db.flush()` force l'écriture.

## Risque / limites attendues
- `insert`/`get` sont **génériques** (`K: AsRef<[u8]>`, `V: Into<IVec>`) : le binder devra figer des signatures concrètes (`bytes`/`str`) — probable `.iri` à la main.
- `IVec` (type opaque) et l'itérateur de tuples enveloppés dans `Result` sont les vrais points durs : exposer un wrapper Iron qui rend des `bytes`.
- Les **transactions** (`db.transaction(|tx| …)`) reposent sur une closure + type d'erreur dédié → hors périmètre, ou wrapper minimal.

## Done
`put`/`get`/`del` corrects et persistants ; `scan_prefix` fonctionne. Cas de test : insérer 5 clés dont 3 sous le préfixe `user:`, **fermer/rouvrir** la base, vérifier que `scan_prefix("user:")` en rend exactement 3.
