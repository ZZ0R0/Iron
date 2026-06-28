# 09 — Sérialisation binaire compacte d'un type
> **Domaine** Données & sérialisation · **Crate(s)** `ironc add bincode` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Sérialise une structure Iron (un enregistrement avec champs + un `Vec`) en binaire compact, l'écrit sur disque, puis la relit et vérifie l'égalité (round-trip). Entrée = une valeur en mémoire ; sortie = un fichier `.bin` + relecture identique.

## Ce que ça teste dans Iron
Le point dur : bincode **ne sérialise pas un type sans dérive** — il exige `Serialize`/`Deserialize` (serde) ou `Encode`/`Decode` (bincode 2). La question testée est donc : **Iron sait-il poser un `#[derive(...)]` sur la struct Rust générée** depuis une classe Iron ? C'est le vrai test de l'interaction « type défini par l'utilisateur ↔ crate qui réclame un trait dérivé ».

## Approche
- Définir le type côté Iron ; obtenir l'équivalent Rust **avec les dérives requises** (mécanisme Iron d'attachement de derive, ou impl écrite à la main dans un `.iri`).
- Encoder : `bincode::serialize(&val)` (v1) ou `encode_to_vec(&val, config::standard())` (v2).
- Décoder : `deserialize` / `decode_from_slice`, puis comparer à l'original.

## Risque / limites attendues
🟡 réel, et **pas** côté binder du crate (l'API bincode est simple) mais côté **dérive du type utilisateur**. Si Iron n'émet pas `#[derive(Serialize, Deserialize)]` sur ses classes, le round-trip d'un type custom est **bloqué** ; replis : sérialiser des primitifs / un `Vec<u8>` assemblé à la main, ou écrire l'impl serde dans un `.iri`. La config bincode 2 est un mini-builder (`config::standard()`). Versions 1 vs 2 = APIs distinctes à figer d'emblée.

## Done
Une valeur encodée puis décodée est **égale** à l'originale (champs + Vec). Cas de test : un enregistrement `{ id: 7, tags: ["a","b"] }` round-trip sans perte, et le fichier binaire est plus court que son équivalent JSON.
