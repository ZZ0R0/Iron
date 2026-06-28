# 65 — Émetteur/vérificateur de JWT
> **Domaine** Crypto & compression · **Crate(s)** `ironc add jsonwebtoken` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Émettre des **JWT** signés (HS256) portant des *claims* (sujet, expiration), et les vérifier : signature, expiration, intégrité. Un jeton trafiqué ou expiré est rejeté.

## Ce que ça teste dans Iron
- L'interaction **classe Iron ↔ `serde`** : les *claims* doivent être (dé)sérialisables — *comment Iron attache-t-il `Serialize`/`Deserialize` à un type utilisateur ?* C'est **le** vrai test ici.
- Le **générique `decode::<Claims>`** paramétré par le type des *claims*.
- `Result` sur la vérification (signature/exp invalides), `EncodingKey`/`DecodingKey` construites depuis un secret en octets.

## Approche
Définir `Claims { sub, exp, iat }`. `encode(header, claims, key)` avec un secret partagé. `decode` avec une `Validation` qui contrôle l'expiration. Montrer trois cas : jeton valide, **signature trafiquée**, **jeton expiré** → les deux derniers en `Err`.

## Risque / limites attendues
- 🟢 sur l'API, **mais** le point sensible est la **dérivation `serde` sur un type Iron** : si le binder/codegen ne sait pas poser les *derives* sur une classe utilisateur, c'est la friction réelle de ce projet (à confirmer honnêtement).
- `Header`/`Validation` sont de petites structs « builder » à champs publics — sans difficulté.
- HS256 seulement ici ; RS/ES (clés asymétriques, PEM) élargiraient la surface et le risque.

## Done
- Jeton émis puis vérifié ; *claims* préservés à l'aller-retour.
- Jeton trafiqué **et** jeton expiré → `Err` typées distinctes, zéro panic.
- Zéro `unsafe`, sortie `std`.
