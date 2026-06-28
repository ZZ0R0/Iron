# 41 — Lecteur audio
> **Domaine** Audio & signal · **Crate(s)** `ironc add rodio` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Lire un fichier WAV ou MP3 depuis le disque et le jouer jusqu'au bout sur la sortie audio par défaut, avec contrôle du volume, pause/reprise et arrêt propre.

## Ce que ça teste dans Iron
- L'import d'un crate qui rend des **handles RAII à garder vivants** : `OutputStream` + son `OutputStreamHandle`, et un `Sink`. Si Iron *droppe* le stream trop tôt (sémantique par valeur, fin de portée), le son se coupe — test direct de la durée de vie des objets retournés.
- Un `Decoder` **générique** sur `Read + Seek` (ouvert sur un `BufReader<File>`), et le trait `Source`.
- Un appel **bloquant** (`sleep_until_end`) plutôt qu'une boucle d'événements — ça, Iron le digère bien.
- rodio démarre en interne son **propre thread audio** (cpal dessous) : le `main` doit rester occupé, sinon le process sort avant la fin du son — cohabitation thread + portée par valeur.

## Approche
`ironc add rodio` puis `import rodio`. Ouvrir le fichier, l'envelopper dans un lecteur bufferisé, construire un `Decoder`, créer le stream de sortie + un `Sink`, `append` la source, régler le volume, puis bloquer jusqu'à la fin. Exposer pause/reprise via les méthodes du `Sink`. Garder explicitement le `OutputStream` vivant pendant toute la lecture. Enchaîner deux pistes dans le même `Sink` (file FIFO) valide la gestion de plusieurs sources.

## Risque / limites attendues
- Le piège classique : `_stream` **doit rester vivant**. À vérifier que la sémantique par valeur d'Iron ne le libère pas en avance (sinon silence immédiat).
- `Decoder` générique + le trait `Source` : le binder auto peut buter ; un `.iri` à la main pour monomorphiser sur `BufReader<File>` est probable.
- `OutputStream` n'est pas `Send` : pas de passage entre threads. Acceptable ici (lecture mono-thread).
- Les erreurs de décodage remontent en `Result` (`DecoderError`) : bon terrain pour le mapping `Result`/`?` d'Iron sur un type d'erreur de crate.

## Done
`ironc run` joue un `.wav` puis un `.mp3` jusqu'au bout, le volume et la pause fonctionnent, et le programme se termine sans couper le son ni paniquer.
