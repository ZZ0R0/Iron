# 44 — Enregistreur micro
> **Domaine** Audio & signal · **Crate(s)** `ironc add cpal hound` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Capturer quelques secondes depuis le micro par défaut et écrire le flux dans un fichier WAV audible.

## Ce que ça teste dans Iron
- **La frontière temps réel.** `build_input_stream` prend un **callback de données** appelé depuis un **thread audio temps réel** détenu par cpal, recevant un `&[f32]`, qui doit être `Send + 'static`.
- Le partage d'**état mutable** entre ce callback et le thread principal : on pousse les échantillons dans un tampon commun (`Arc<Mutex<Vec<f32>>>`) — exactement l'**état mutable partagé** qui cogne avec « sémantique par valeur, pas d'état global mutable ».
- La négociation de config (format `f32`/`i16`/`u16` → `match`) et le handle de stream `!Send`.
- Le `Stream` doit rester **vivant** pendant la capture (le drop l'arrête) : encore un handle RAII à garder en vie sous sémantique par valeur.

## Approche
`ironc add cpal hound`. Récupérer le host, le device d'entrée par défaut, la config supportée ; construire un stream d'entrée dont le callback copie les échantillons dans un tampon partagé ; `play()`, attendre N secondes, libérer le stream ; écrire le tampon en WAV via hound. À défaut d'`Arc<Mutex>` ergonomique, replier sur un `channel` (mpsc) : le callback envoie, le thread principal draine — motif peut-être plus digeste pour Iron.

## Risque / limites attendues
- **Mur attendu.** La signature à deux closures de `build_input_stream` (données + erreur), générique sur le type d'échantillon, est rude pour le binder.
- Surtout, la closure doit **capturer un état mutable partagé** (`Arc<Mutex>`) et traverser une frontière de thread temps réel : les lambdas par valeur d'Iron et l'absence d'état mutable partagé rendent ce motif très inconfortable, voire inexpressible sans `.iri` lourd.
- Idéalement aucune allocation dans le callback — contrainte temps réel qu'Iron n'aide pas à respecter.
- Le format négocié peut ne pas être `f32` : prévoir la conversion `i16`/`u16 → f32` avant l'écriture WAV.

## Done
Capture N secondes du micro et produit un WAV au contenu audible. **Honnêtement** : blocage probable à la frontière callback / état partagé ; documenter précisément où ça coince.
