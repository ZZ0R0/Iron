# 10 — Parseur XML (extraction de champs)
> **Domaine** Données & sérialisation · **Crate(s)** `ironc add quick-xml` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Lit un fichier XML en streaming et extrait des champs ciblés (par ex. tous les `<title>` et l'attribut `id` de chaque `<item>`). Entrée = `.xml` ; sortie = les valeurs extraites (liste, ou petit JSON/CSV).

## Ce que ça teste dans Iron
quick-xml est un **pull-parser** : une boucle qui appelle `read_event()` et **matche un enum `Event`** (`Start`, `End`, `Text`, `Empty`, `Eof`…) portant des **tranches d'octets empruntées** (`BytesStart`, `BytesText`). On teste donc le `match` sur un enum non trivial, la gestion d'un **état de parsing** (sans état global mutable), et le décodage `&[u8]` → `str`.

## Approche
- `Reader::from_str(&xml)` (ou `from_reader`), boucle `loop { match reader.read_event() { … } }`.
- Sur `Event::Start`, comparer `e.name()` ; lire les attributs via `e.attributes()` (itérateur de `Result<Attribute>`).
- Sur `Event::Text`, décoder (`e.unescape()` / `String::from_utf8`) et accumuler ; sortir sur `Eof`.

## Risque / limites attendues
🟡. L'API n'est pas un builder mais elle est **fiddly pour le binder** : `Event` transporte des données **empruntées avec durées de vie** liées au buffer du reader, et le pattern recommandé `read_event_into(&mut buf)` réutilise un tampon mutable — peu naturel en sémantique par valeur. Les noms/textes sont des `&[u8]` (pas `str`) → décodage explicite. Gérable, mais attendre des frictions sur lifetimes/emprunts ; un `.iri` simplifiant peut aider.

## Done
Sur un XML à plusieurs `<item id="..">`, extrait la liste des `id` et des `<title>` dans l'ordre du document. Cas de test : `<r><item id="1"><title>A</title></item></r>` produit `id=1, title=A`.
