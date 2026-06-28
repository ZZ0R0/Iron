# 16 — Client HTTP qui interroge une API et parse le JSON
> **Domaine** Web & réseau · **Crate(s)** `ironc add ureq serde_json` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Un client en ligne de commande qui appelle une API REST publique (ex. JSONPlaceholder/httpbin), récupère la réponse et en extrait des champs. En entrée : une URL ; en sortie : quelques valeurs parsées (ex. titre + id d'un post), avec gestion des erreurs réseau et des codes HTTP ≠ 200.

## Ce que ça teste dans Iron
La combinaison la plus courante du web : un **client HTTP synchrone** (ureq) + un **parseur JSON dynamique** (serde_json `Value`). Teste la navigation dans un `Value` via l'**opérateur d'indexation** (`v["champ"]`) et les accesseurs `as_str()/as_u64()` rendant des `Option`.

## Approche
- `ureq::get(url).call()?` → `Response` ; `.into_string()?` → corps texte (ou `.status()` pour le code).
- `serde_json::from_str::<Value>(&body)?` → `Value`.
- Navigation : `v["title"].as_str()`, `v[0]["id"].as_u64()`, `v.get("x")` ; itérer un tableau via `as_array()`.

## Risque / limites attendues
- Très faible. Point d'attention : `into_json::<T>()` exige un type cible → rester sur `Value` (pas de `#[derive(Deserialize)]` en Iron).
- L'**indexation `Value`** suppose qu'Iron mappe `[]` vers `Index` ; sinon utiliser `.get(k)`/`.pointer()`. Les `Option` en chaîne demandent un déballage soigné (Result/Option d'Iron).
- Erreurs ureq : distinguer **transport** (DNS/timeout) et **statut HTTP** (`ureq::Error::Status`) — deux branches à gérer.

## Done
GET réel sur une API publique, parse OK, champs extraits affichés ; une URL injoignable rend une erreur propre (pas de panic). Cas de test : GET `/posts/1`, vérifier `id == 1` et `title` non vide ; hôte injoignable → message d'erreur + code de sortie ≠ 0.
