# planify-5.1 — Stratégie **io** : `Read`→source, `Write`→puits

- Réalise (cadrage) : [52800]
- Version : V12 · Vague : 5 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (pont)

## But
Matérialiser les bornes **io** sur le pont (RACINE 1) : un param `R: Read` accepte des bytes/un chemin Iron ; un
param `W: Write` accepte un **buffer** rendu en retour. Débloque quick-xml, gpx, flate2, csv-reader, image-stream.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (entrées `STRATEGIES` pour `Read`/`BufRead`/`Write`),
  `ironc/src/lower.rs` (émission des wrappers).
- Séquentielle (vague 5).

## Interface produite / consommée
- **Consomme** : `bridge::{accepts, materialize, STRATEGIES}` (1.1).
- **Produit** : appels io de crates sans fournir d'objet Rust à la main.

## Étapes
1. **Read (entrée).** `accepts(Read) = str|list[u8]` ; `materialize` : `std::io::Cursor::new(bytes)` pour des
   octets, `std::fs::File::open(path)?` pour un chemin (heuristique : type Iron source). BufRead → `BufReader`.
2. **Write (sortie) — cas mémoire/fichier.** `accepts(Write) = ` sortie spéciale ; `materialize` : `Vec<u8>` (ou
   `File::create`), passé `&mut`, puis **rendu** comme `list[u8]`/écrit sur disque. Émettre le glue de retour.
3. Router via `try_extern_*` (lower) en consultant le pont — pas de `match` crate-spécifique.

## Definition of Done (vérifiable)
1. quick-xml/gpx : lecture d'un document depuis une **str/bytes** Iron (run réel) ; gpx parse une trace.
2. flate2 : `compress(bytes)`→`list[u8]`→`decompress` round-trip (Write→buffer rendu).
3. **Gate VERT** ; 0 `unsafe`.

## Notes
- **Friction connue** : le streaming continu (`Write` au fil de l'eau, gros volumes) reste **partiel** au premier
  jet — on couvre buffer-mémoire + fichier. À noter pour itération, pas à survendre.
