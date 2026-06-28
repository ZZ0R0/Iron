# 14 — Client Redis (cache get/set)
> **Domaine** Bases de données · **Crate(s)** `ironc add redis` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un cache simple au-dessus d'un Redis local : `set clé valeur [ttl]`, `get clé`, `del clé`. En entrée : une commande + un Redis joignable (`redis://127.0.0.1/`) ; en sortie : la valeur en cache ou un « miss ». Le TTL est respecté (expiration).

## Ce que ça teste dans Iron
Le binding d'un client réseau dont l'API haut niveau (`Commands`) est **générique à polymorphisme de retour** (`set<K,V,RV>`, `get<K,RV>` via `FromRedisValue`). Teste si Iron peut soit consommer ce trait, soit retomber sur l'API **bâtie en commandes** (`cmd().arg().query()`), plus plate.

## Approche
- `redis::Client::open("redis://127.0.0.1/")?` → `Client` ; `client.get_connection()?` → `Connection`.
- Voie haute (générique) : `con.set(k, v)`, `con.get::<_, String>(k)` (trait `Commands`).
- Voie basse (recommandée pour le binder) : `redis::cmd("SET").arg(k).arg(v).query(&mut con)` ; TTL via `cmd("SETEX")` ou `EXPIRE`.

## Risque / limites attendues
- Le trait `Commands` exige des **bornes `ToRedisArgs`/`FromRedisValue`** et un **type de retour choisi par turbofish** : difficile à exposer proprement → privilégier `cmd().arg().query()`, plus bindable, quitte à perdre l'ergonomie.
- `Connection` est **mutable** (`&mut`) : cogne avec la sémantique par valeur ; l'encapsuler dans une classe Iron qui possède la connexion.
- Variante **async** (`tokio`) à éviter ici : rester sur le client **synchrone**. Nécessite un Redis lancé (infra de test).

## Done
`set`/`get`/`del` corrects contre un Redis local. Cas de test : `set foo bar` puis `get foo == bar` ; `del foo` puis `get foo == miss` ; `set` avec TTL=1 s, `get` après 1,5 s → miss.
