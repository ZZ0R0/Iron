# 21 — Serveur de chat TCP multi-clients
> **Domaine** Réseau bas-niveau · **Crate(s)** `import std` — `net`, `thread` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un serveur TCP qui accepte plusieurs clients en parallèle ; chaque message reçu d'un client est **rediffusé** à tous les autres (chat broadcast). Un client = un simple `nc`/telnet. Objectif : valider le trio `TcpListener` / `TcpStream` / threads sur un cas où **plusieurs threads partagent un état mutable** (le registre des clients connectés).

## Ce que ça teste dans Iron
- `import std` pour `net` (`TcpListener.bind`, `incoming`, `TcpStream`) et `thread.spawn` — API « feuille » que le binder/codegen doivent exposer proprement.
- Le **cœur du test** : le registre des connexions est un **état mutable partagé entre threads**. Or Iron est *par valeur* (`y = x` copie) et *sans état global mutable*. Partager le registre exige un `ref` explicite (I3 : un alias mutable ne se résout **jamais** par auto-clone, sinon les copies divergent et le broadcast ne touche plus personne).
- La closure passée à `thread.spawn` **capture** l'environnement et impose des bornes `Send + 'static` : test direct de la capture + des bornes que `rustc` fera respecter (I2, filet).

## Approche
Boucle d'`accept` dans le thread principal ; un thread par client. Préférer le **passage de messages** (`mpsc`) au verrou partagé : chaque client publie sur un canal, un thread « diffuseur » relaie — modèle qui colle mieux à la sémantique par valeur qu'un `Arc<Mutex<Vec<…>>>`. Le registre des `Sender` reste néanmoins partagé et doit être annoté `ref`.

## Risque / limites attendues
🟡 : l'API net + threads est simple, mais l'**état partagé mutable** force des annotations explicites (`ref` → `Arc<Mutex<…>>`) et les **closures de thread** déclenchent les bornes `Send`/`'static` que le résolveur optimiste peut rater (erreur `rustc` remontée en source Iron, I2). Un petit `.iri` ou un `ref` bien placé suffit ; pas de mur, juste de la friction sur le partage cross-thread.

## Done
Deux clients `nc` connectés ; un message tapé par l'un apparaît chez l'autre ; la déconnexion d'un client ne crashe ni le serveur ni les autres sessions. Build `cargo` vert sur le Rust généré.
