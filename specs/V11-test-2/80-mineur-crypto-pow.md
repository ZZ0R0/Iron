# 80 — Mineur de cryptomonnaie jouet (preuve de travail)
> **Domaine** Langages & exotiques · **Crate(s)** `ironc add sha2` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Un mineur PoW jouet : construire un bloc (index, hash précédent, données, nonce), le hacher en SHA-256 et
incrémenter le nonce jusqu'à obtenir un hash avec N zéros en tête (la « difficulté »). Mesurer le hashrate.

## Ce que ça teste dans Iron
- `sha2` (RustCrypto), API propre déjà éprouvée ailleurs : `Sha256::new()` → `.update(bytes)` →
  `.finalize()` → `[u8; 32]` ; seul petit grain de sable, le retour `GenericArray` à mapper ;
- une **boucle entière serrée** (perf) incrémentant le nonce — bon test de la qualité du Rust généré par
  Iron sur du calcul chaud (le `TokenStream` émis doit rester proche d'un mineur écrit à la main) ;
- **inspection d'octets/bits** pour compter les zéros de tête et **formatage hexadécimal** du hash ;
- en bonus, **`std::thread`** pour répartir des plages de nonce — sans état global mutable (chaque thread
  sa plage, renvoi à la trouvaille).

## Approche
Sérialiser l'en-tête du bloc en octets, hacher, comparer aux N zéros requis, sinon `nonce += 1` et
recommencer. Garder le cœur **mono-thread** (🟢) ; le multi-thread est un extra pour exercer `thread::spawn`
+ partage **immuable** de la cible entre workers, cohérent avec « pas d'état global mutable ».

## Risque / limites attendues
🟢 — aucun mur : `sha2` est simple et la logique est pure. Les seuls signaux à surveiller : le mapping
`GenericArray`/`[u8; 32]` et la **performance de la boucle chaude**. Si le hashrate Iron s'effondre face au
Rust manuel, c'est un signal perf à remonter (clone-fallback ?), pas un blocage.

## Done
Mineur qui trouve un nonce satisfaisant une difficulté réglable (ex. 4 zéros hexa), affiche le hash gagnant
+ le nonce + le hashrate, sur quelques blocs enchaînés. Variante multi-thread optionnelle.
