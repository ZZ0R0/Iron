# 19 — API REST (routes GET/POST, async)
> **Domaine** Web & réseau · **Crate(s)** `ironc add axum tokio` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Une petite API REST en mémoire : `GET /items` (liste), `POST /items` (crée depuis un JSON), `GET /items/:id`. En entrée : des requêtes HTTP ; en sortie : du JSON + des codes HTTP corrects, le tout servi par un runtime **async** (tokio) avec un état partagé en mémoire.

## Ce que ça teste dans Iron
Le haut du mur web : **runtime async (tokio)** + **routeur à extracteurs** dont l'enregistrement repose sur un **trait `Handler` blanket** sur l'arité des fonctions et des traits d'extraction (`FromRequest`, `IntoResponse`). Teste l'async d'Iron **et** sa capacité à consommer une API « magique » par traits.

## Approche
- `#[tokio::main]` + `Router::new().route("/items", get(list).post(create))` ; `.with_state(state)`.
- Handlers `async` : `async fn create(State(s): State<AppState>, Json(body): Json<Item>) -> impl IntoResponse`.
- Extracteurs : `State`, `Path`, `Json` ; état partagé `Arc<Mutex<Vec<Item>>>`.
- Lancement : `axum::serve(listener, router).await`.

## Risque / limites attendues
- L'enregistrement de handlers via **impl blanket sur l'arité + traits d'extraction** est quasi inbindable automatiquement : il faudra probablement un **wrapper Iron étroit** exposant un sous-ensemble de routes, ou renoncer à l'ergonomie des extracteurs.
- `#[tokio::main]` (macro) et `IntoResponse` (retour polymorphe) sont des points durs ; mapper l'async d'Iron sur tokio reste à prouver à cette charge.
- **État partagé mutable** = `Arc<Mutex<…>>` : frontal avec « pas d'état global mutable » d'Iron → l'état doit être **passé** explicitement, pas global.

## Done
Même réduite : `POST /items` crée, `GET /items` reliste l'élément, `GET /items/:id` le retrouve, codes 200/201/404 corrects, en async. Cas de test : POST `{"name":"x"}` → 201 ; la liste contient `x` ; GET `/items/99` → 404. **Documenter ce qui n'a pas pu passer par le binder.**
