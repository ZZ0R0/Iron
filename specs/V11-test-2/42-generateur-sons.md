# 42 — Générateur de sons / petit synthé
> **Domaine** Audio & signal · **Crate(s)** `ironc add hound` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Synthétiser des formes d'onde (sinus, accords, enveloppe simple) en mémoire et les écrire dans un fichier WAV valide, lisible par n'importe quel lecteur.

## Ce que ça teste dans Iron
- Une API **feuille, presque idéale** pour le binder : `WavWriter::create(path, spec)` + `write_sample(i16)`, et `WavSpec` comme simple struct de champs.
- La **surcharge d'opérateurs** d'Iron sur un type échantillon/oscillateur : additionner des sinus pour un accord, multiplier par une enveloppe — du DSP par valeur, sans état partagé.
- Les boucles de génération de buffers et la conversion `f32 → i16` (quantification, clipping) écrites par le dev.
- L'écriture **streaming** échantillon par échantillon (pas un buffer d'un coup) : la boucle tient un `WavWriter` mutable ouvert — `mut` simple, sans état partagé.

## Approche
`ironc add hound`. Définir une classe oscillateur (fréquence, phase) qui produit des échantillons ; surcharger `+` pour mixer plusieurs voix en un accord, `*` pour appliquer une enveloppe. Remplir un `WavSpec` (mono, 44100 Hz, 16 bits), créer le `WavWriter`, écrire les échantillons quantifiés, `finalize()`. Tester aussi un format **stéréo** (entrelacement L/R) pour vérifier l'écriture alternée des canaux.

## Risque / limites attendues
- Risque faible : tout est concret et par valeur. `WavWriter` est générique sur `Write`/`Seek` et `Sample`, mais utilisé sur `File`/`i16` il se monomorphise sans douleur.
- Seul point d'attention : la **quantification** et le **clipping** (saturer plutôt qu'envelopper) sont la responsabilité du dev, pas d'Iron — une erreur ici fait un WAV qui craque, pas une panne du langage.
- `finalize()` doit être appelé pour réécrire les tailles dans l'en-tête : l'oublier produit un WAV tronqué — à ne pas perdre dans la sémantique de fin de portée.

## Done
Génère un sinus pur, un accord à trois voix et une note avec enveloppe ; le WAV produit est valide et s'écoute correctement dans un lecteur tiers.
