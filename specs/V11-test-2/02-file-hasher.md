# 02 — Hacheur de fichiers (checksums récursifs d'un dossier)
> **Domaine** CLI & fichiers · **Crate(s)** `ironc add sha2` `ironc add walkdir` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Parcourt récursivement un dossier et calcule le SHA-256 de chaque fichier, façon `sha256sum -r`. Entrée = un chemin de dossier ; sortie = lignes `<hex>  <chemin>` triées, éventuellement écrites dans un fichier manifeste.

## Ce que ça teste dans Iron
Deux crates « feuilles » à API simple, combinés. sha2 expose le trait `Digest` (méthodes `new`/`update`/`finalize`) ; walkdir un itérateur de fichiers. On valide la **résolution de méthodes de trait**, l'**itération sur des `Result`**, et la lecture binaire std — un socle réutilisé partout ensuite.

## Approche
- walkdir : `WalkDir::new(dir).into_iter()`, filtrer les fichiers, gérer chaque `Result<DirEntry>`.
- sha2 : `Sha256::new()`, `hasher.update(bytes)` (lecture par bloc ou fichier entier via std), `hasher.finalize()`.
- Le digest est un `GenericArray<u8>` → formatage hexadécimal (boucle `{:02x}` ou crate `hex`).

## Risque / limites attendues
Faible. Points d'attention : le trait `Digest` doit être **en scope** pour que `update`/`finalize` résolvent (le binder doit exposer l'impl de trait) ; la sortie `finalize()` est un type **générique** (`GenericArray`) qu'Iron doit savoir traiter comme un `[u8]` pour le formatage. walkdir peut renvoyer des erreurs de permission à ignorer proprement. Rien de bloquant → 🟢.

## Done
Sur un dossier de test à 3 fichiers, produit 3 lignes `hex  chemin` ; le hash d'un fichier connu correspond à `sha256sum`. Cas de test : un fichier contenant `abc` doit donner `ba7816bf8f01cfea...`.
