# 62 — Chiffreur/déchiffreur de fichiers
> **Domaine** Crypto & compression · **Crate(s)** `ironc add chacha20poly1305` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un outil `encrypt <fichier>` / `decrypt <fichier>` qui chiffre n'importe quel fichier avec **XChaCha20-Poly1305** sous une phrase de passe. Toute altération du fichier chiffré est détectée au déchiffrement.

## Ce que ça teste dans Iron
- **IO fichier brut** (`import std`) : lire/écrire des octets binaires, pas du texte.
- Le **trait `Aead`** une 2ᵉ fois, sur un autre crate de la même famille (RustCrypto) — cohérence du binder.
- **`Result` pour l'intégrité** : tag Poly1305 invalide ⇒ `Err`, jamais de sortie corrompue silencieuse.
- Tampons `Vec<u8>` volumineux → bon révélateur du **clone-fallback** d'Iron (warning perf `--audit`, I4).

## Approche
Phrase → clé 256 bits (KDF simple ou `argon2`). Nonce **24 octets** aléatoire (XChaCha tolère le tirage aléatoire sans compteur). On chiffre le contenu en un coup, on écrit `nonce ‖ ciphertext`. Le déchiffrement relit le nonce, vérifie le tag, restitue les octets. Pour rester 🟡 on évite l'API **streaming** (encore plus générique).

## Risque / limites attendues
- 🟡 Mêmes frictions que 61 : `GenericArray`, trait `Aead`, constructeur `new(key)` — un `.iri` manuel peut être requis.
- **Tout en mémoire** : un gros fichier passe entier en RAM ; le découpage en chunks impose l'API streaming, plus dure pour le binder.
- Pas de gestion de clé « pro » (ni rotation, ni format type `age`) — volontairement minimal.

## Done
- `encrypt` puis `decrypt` d'un fichier binaire → **octet-pour-octet identique**.
- Un octet retourné dans le chiffré → `Err` au déchiffrement, zéro panic.
- Aucun `unsafe`, sortie 100 % `std`.
