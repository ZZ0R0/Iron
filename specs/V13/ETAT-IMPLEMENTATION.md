# V13 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3)

> Réalisation des 5 vagues. **Honnête** : 2 crates **complétés** (chrono, midly) + 3 **mécanismes** livrés
> (octets, `@write`, `display`) qui aident sans toujours **finir** leur cible. La classe **dimensions matrices**
> est un **mur** (crate-spécifique). Gate VERT 24/0 à chaque étape, tout poussé.

## Résultat par vague

| Vague | Spec | Mécanisme livré | Crate(s) | Résultat |
|:--:|---|---|---|---|
| 1 | [24500]A octets | `GenericArray<u8>`/`Key`/`Nonce` → `@arr` (clé matérialisée) | aes-gcm, chacha20 | **🟡 partiel** : clé OK, mais **AEAD-trait** (`encrypt`/`decrypt` génériques droppés) + **alias-cipher** `Aes256Gcm` non utilisable bloquent l'e2e |
| 2 | [30200] N1 | idiome `display` (retour `Display`-borné → `.to_string()`) | **chrono** | **✅** : `date.format("%Y-%m-%d")` → `2026-06-29`. chrono COMPLET. |
| 3 | [52800]w | sentinelle `@write` (puits→fichier, fn libre) | flate2, printpdf | **🟡 partiel** : `serde_json::to_writer(path, v)` ✅ ; mais flate2 = **stream/builder** (pas une fn à puits), printpdf = **méthode + doc complexe** → @write-méthode = follow-up |
| 4 | [24500]B dims | — | nalgebra, ndarray… | **🔴 mur** : ctors **par-dimension** (`Vector3::new`) + **collision** `Matrix.new` + table d'alias = **crate-spécifique** (interdit). Non faisable globalement. |
| 5 | [30200] N2 | (aucun code — pattern) | **midly**, quick-xml | **✅ midly** via `Smf.parse(raw).make_static()` (`tracks: 1`) — le crate fournit la conversion possédée, l'utilisateur la chaîne. quick-xml(events) idem `into_owned` (probable). |

## Bilan V13
- **+2 crates complétés** : **chrono** (format), **midly** (parse via make_static).
- **3 mécanismes** réutilisables livrés : octets-`@arr`, idiome `display`, `@write` (fn libre).
- **Murs honnêtes** : **dimensions matrices** (nalgebra/ndarray/rustfft/polars/rapier2d/smartcore) — ctors
  par-dimension + collision, **non globalisable sans noms de crate** ; **AEAD** (aes-gcm/chacha20) — trait
  générique `Aead` ; **stream** (flate2) — API builder. Ces 3 familles dépassent une simple sous-feature.

## Plafond réel après V13
~30 (V12) + **chrono déjà compté** + **midly (+1)** = **~31 fermes sans matériel** ; aes-gcm/flate2/printpdf
restent **partiels** (mécanisme posé, e2e bloqué) ; la classe **dims (~6-8)** reste **murée**. Donc le « ~37-39 »
visé n'est pas atteint : **les dimensions matrices sont un vrai mur**, pas un polish. Je le dis net.

## Ce qu'il faudrait pour aller plus loin (hors V13, honnête)
- **AEAD** (aes-gcm/chacha20) : matérialiser les **méthodes de trait génériques** (`encrypt(nonce, msg)`) +
  binder l'**alias-cipher** comme type — gros.
- **dims matrices** : un mécanisme **non crate-spécifique** pour les ctors par-dimension — pas trouvé ; possiblement
  un mur de conception (les ctors par-dim ne sont pas une généralité exprimable sans connaître les alias).
- **stream io** (flate2) : modèle de **flux** (encoder/decoder builder) — au-delà du puits one-shot.
