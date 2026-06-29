# planify-3.1 — io-écriture (puits `Write`, sentinelle `@write`) → flate2, printpdf

- Réalise (cadrage) : [52800]w
- Version : V13 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : — (parallèle conceptuel de `@read`, V12 [52800])

## But
Matérialiser un **puits `W: Write`** : vers **fichier** (printpdf `doc.save(path)`) ou **buffer rendu** (flate2
compresse → `list[u8]`). Symétrique de `@read` (lecture).

## Fichiers POSSÉDÉS
- `ironc/src/bind.rs` : `bound_to_iron` (`Write`/`BufWrite` → `@write`) ; `project` (param `@write`) ; `render`
  (chemin-closure : fichier `BufWriter::new(File::create(path))` ou buffer rendu `{ let mut __b=Vec::new();
  ORIG(&mut __b,…); __b }`).
- `ironc/src/lower.rs` : `try_extern_method` — un arg `@write` d'une **méthode** (`doc.save(w)`) → matérialisé
  au site d'appel (`&mut BufWriter::new(File::create(path).unwrap())`).

## Interface produite / consommée
- **Consomme** : la machinerie `wraps`/chemin-closure de V12.
- **Produit** : fns/méthodes à puits acceptant un **chemin** (fichier) ou rendant un `list[u8]` (buffer).

## Étapes
1. `bound_to_iron` : `Write`/`BufWrite` → `@write`.
2. `project` : `@write` → param `str` (chemin) ; choisir saveur (fichier vs buffer rendu) par le **type de
   retour** de la fn (`-> ()` + puits unique → buffer rendu `list[u8]` ; sinon fichier).
3. `render` : étendre le chemin-closure pour `@write` (fichier / buffer rendu) — comme `@read`/`@arr` (clé `wraps`).
4. `lower::try_extern_method` : arg `@write` en position méthode → `&mut BufWriter::new(File::create(path).unwrap())`.
5. Régénérer flate2/printpdf, tester.

## Definition of Done (vérifiable)
1. flate2 : `compress(bytes)` → `list[u8]` puis `decompress` round-trip = entrée (run réel ; lecture déjà OK V12).
2. printpdf : `doc.save("/tmp/out.pdf")` écrit un fichier PDF non vide (run réel, vérifié `file`).
3. Gate VERT 24/0 ; 0 `unsafe`.

## Notes
- Flux continu (gros volumes, flush répétés) **hors périmètre** — fichier/buffer one-shot seulement (noter).
- Méthode + puits **buffer** (rare) : si rencontré, prototyper ; sinon noter.
