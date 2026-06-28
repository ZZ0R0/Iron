# 61 — Gestionnaire de mots de passe (coffre chiffré AES)
> **Domaine** Crypto & compression · **Crate(s)** `ironc add aes-gcm` `ironc add argon2` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un coffre CLI : une **phrase maîtresse** déverrouille un fichier chiffré contenant des couples (service, identifiant, secret). Commandes `add`, `get`, `list`. Le fichier au repos est inexploitable sans la phrase.

## Ce que ça teste dans Iron
- **Deux crates qui coopèrent** : `argon2` dérive une clé de 32 octets depuis la phrase + un sel ; `aes-gcm` chiffre avec cette clé.
- **`Result`/`Option` sur de la crypto faillible** : une mauvaise phrase = déchiffrement qui **échoue proprement** (tag d'authentification invalide), pas un panic.
- **Buffers d'octets** : `Vec<u8>`, tableaux de taille fixe (clé 32, nonce 12), conversion str↔octets — la sémantique par valeur d'Iron sur des tampons.
- Génération aléatoire (sel, nonce) via un RNG du système.

## Approche
Phrase → `argon2` (sel aléatoire stocké en clair) → clé 256 bits. Pour chaque entrée : nonce aléatoire, chiffrement **AES-256-GCM**, on sérialise `sel ‖ nonce ‖ ciphertext` dans le fichier coffre (JSON ou binaire simple). `get` redérive la clé et déchiffre. Le nonce **doit** être unique par chiffrement — invariant de sécurité qu'Iron ne vérifie pas pour toi.

## Risque / limites attendues
- 🟡 `aes-gcm` expose ses tailles via des **types génériques** (`GenericArray`, constantes de niveau type) et un **trait `Aead`** : le binder auto risque de mal rendre ces signatures → un `.iri` à la main pour `new(key)` / `encrypt(nonce, data)` peut être nécessaire.
- Le RNG (`OsRng`) vient d'un crate tiers (`rand_core`/`getrandom`) : dépendance implicite à exposer.
- Aucun `unsafe`, que du `std` : OK. La **réutilisation de nonce** reste un piège logique, hors du filet d'Iron.

## Done
- Coffre créé, entrée ajoutée, relue après réouverture du fichier.
- Mauvaise phrase maîtresse → `Err` typée, **zéro panic**, zéro `unsafe`.
- Le fichier au repos ne révèle aucun secret en clair.
