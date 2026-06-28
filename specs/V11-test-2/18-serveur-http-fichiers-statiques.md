# 18 — Serveur HTTP de fichiers statiques
> **Domaine** Web & réseau · **Crate(s)** `ironc add tiny_http` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Un serveur web minimal qui sert les fichiers d'un dossier. En entrée : un port + une racine ; en sortie : pour chaque requête GET, le fichier correspondant (200 + bon `Content-Type`) ou une 404. Boucle bloquante mono-process, sans async.

## Ce que ça teste dans Iron
Un serveur réseau **sans runtime async** : une **boucle bloquante** sur un itérateur de requêtes. Teste l'I/O synchrone, la lecture de fichiers (`std`), la construction de réponses et d'en-têtes, et le mapping URL→chemin (avec garde anti-`..`).

## Approche
- `Server::http("0.0.0.0:8000")?` → `Server`.
- Boucle : `for req in server.incoming_requests()` (bloquant) ; `req.url()`, `req.method()`.
- Réponse : `Response::from_file(File)` ou `Response::from_string(s)` ; en-tête `Content-Type` via `Header::from_bytes(...)` ; `req.respond(resp)`.
- Résolution : joindre racine + URL, refuser la remontée `..`, deviner le MIME à l'extension.

## Risque / limites attendues
- `Response<R>` est **générique sur `Read`** : les constructeurs (`from_file`/`from_string`) donnent des types concrets, mais une fonction qui renvoie « une réponse quelconque » bute sur le paramètre générique → fixer le type ou brancher tôt.
- `Header::from_bytes` rend un `Result` un peu rugueux ; table d'extensions→MIME à la main.
- Mono-thread : une requête lente bloque les autres (acceptable ici) ; sinon un thread par requête (`std::thread`).

## Done
Sert un dossier réel : GET d'un fichier existant → 200 + contenu exact + `Content-Type` correct ; chemin inexistant → 404 ; tentative `../` → refus. Cas de test : `curl` un `index.html` (200, corps égal) et `/nope` (404).
