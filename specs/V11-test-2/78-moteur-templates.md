# 78 — Moteur de templates (type Jinja)
> **Domaine** Langages & exotiques · **Crate(s)** `—` *(à la main, `std` seul)* · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Un moteur de gabarits inspiré de Jinja : substituer `{{ variable }}`, gérer `{% if %}` et `{% for %}`,
rendre un template avec un contexte de données. **Aucun crate** — 100 % Iron + `std`.

## Ce que ça teste dans Iron
La vitrine « logique pure » d'Iron, sans aucune FFI :
- **manipulation de chaînes** : scan/découpe pour repérer `{{ }}` et `{% %}`, avec le piège des
  **frontières UTF-8** (Rust slice par octets, pas par char) — Iron doit offrir un slicing sûr et ergonomique ;
- un **AST récursif** (`Node::Text | Var | If | For`) et la **récursion de rendu** ;
- un **contexte** `HashMap<String, Value>`, avec `Value` possiblement imbriqué (listes pour `for`) sous
  sémantique par valeur ;
- ni async, ni matériel, ni dépendance : tout repose sur le langage lui-même et sa lib standard.

## Approche
Parser le template en une liste de nœuds (texte littéral + balises), puis **rendre récursivement** en
consultant le contexte. Garder un `Value` modeste (chaîne, nombre, booléen, liste) suffisant pour `if` et
`for`. Gérer proprement la variable absente et la balise mal formée plutôt que de paniquer.

## Risque / limites attendues
🟢 — aucun mur : le seul stress est l'**ergonomie du slicing de chaînes** (octets vs chars) et l'**enum
récursif**. Si Iron tient ces deux points, c'est vert franc ; sinon c'est un signal précieux sur sa
manipulation de `String`/`&str` — exactement ce qu'un test de stabilité doit faire ressortir.

## Done
Moteur qui rend un gabarit non trivial (texte + `{{ var }}` + au moins un `{% for %}` ou `{% if %}`) avec
un contexte donné et produit la sortie attendue, en gérant les cas d'erreur basiques.
