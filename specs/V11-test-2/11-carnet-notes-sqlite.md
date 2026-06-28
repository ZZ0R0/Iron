# 11 — Carnet de notes SQLite (CRUD)
> **Domaine** Bases de données · **Crate(s)** `ironc add rusqlite` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Un CLI de prise de notes adossé à un fichier SQLite. En entrée : une commande (`add "texte"`, `list`, `update <id> "texte"`, `rm <id>`) ; en sortie : la note créée/modifiée ou la liste des notes persistées. La base survit entre deux exécutions.

## Ce que ça teste dans Iron
Le binding d'une API « feuille » classique mais centrale : ouverture de connexion, `execute` paramétré, et surtout la lecture de lignes via `query_map`, qui prend une **closure de mapping** `Row -> T`. Cas d'école pour valider le passage de paramètres typés (`&dyn ToSql`) et le rapatriement de colonnes.

## Approche
- `Connection::open(path)` → `Connection`.
- `conn.execute("CREATE TABLE IF NOT EXISTS …", [])` ; insert/update/delete via `execute(sql, params)`.
- Lecture : `conn.prepare(sql)` → `Statement`, puis `stmt.query_map(params, |row| row.get(0))` → itérateur de `Result<T>`.
- `row.get::<usize, i64/String>(idx)` par colonne ; agréger en une liste côté Iron (struct `Note { id, body, created }`).

## Risque / limites attendues
- Le macro `params![…]` de rusqlite n'existe pas en Iron : passer les paramètres en **tableau/tuple** (`[&body]`) ou via `&[&dyn ToSql]` — un `.iri` à la main peut être nécessaire pour le trait objet `ToSql`.
- La closure de `query_map` + l'itérateur de `Result` sont le seul point sensible ; tout le reste est plat.
- Pas de pool ni d'async : monothread, suffisant ici.

## Done
CRUD complet vert et **persistant**. Cas de test : créer 3 notes, en supprimer une, **fermer puis rouvrir** la base, vérifier que `list` rend exactement 2 notes avec les bons corps.
