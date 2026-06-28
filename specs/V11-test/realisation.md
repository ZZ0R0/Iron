# V11-test — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert).
> Benchmark **CPython vs Iron→Rust** sur ~30 programmes compute-bound. PoC `factor` ~21× déjà prouvée.

## Avancement des vagues

| Vague | Piste | Titre | Statut |
|---|---|---|---|
| 1 | 1.1 | Fondation : `bench.sh` + datasets + contrat + `factor` e2e | ✅ fait |
| 2 | 2.1 | Prélude idiomes Iron (split()/slice−/join/ord/chr/count/char/…) | ✅ fait |
| 3 | 3.1–3.16 | Fan-out bucket A (wc grep uniq cut tr nl fold rev tac comm seq od expand cksum factor sort) | ✅ fait (16/16) |
| 4 | 4.1–4.14 | Fan-out bucket B (wordfreq sieve histogram naivesort csvsum rle jsonsum collatz editdist matmul gcdsum primecount dotproduct anagram) | ✅ fait (14/14) |
| 5 | 5.1 | Unification + verdict (tableau perf, speedup géométrique, trous) | ✅ fait (30/30 · 14.73×) |
| 6 | 6.1 | Bonus exotique threading (Rust vs GIL) — **optionnel** | ⏸️ proposé (sur demande) |

## Invariants de réalisation

- **Zéro type manuel** des deux côtés (`.py` ET `.irn`). L'inférence Iron fait tout.
- `.irn` = **même corps** que `.py`, **seuls imports/entrée rebindés**. Logique identique.
- **Compute-bound** : mode résumé (1 ligne) + boucle chaude en **langage** (pas de lib).
- Chaque programme : **même sortie** py3↔iron · **0 `unsafe`** · ligne de bench.
- Modifs `ironc/src` **uniquement** en vagues séquentielles (2.1, touch-ups 5). Fan-out = dossiers disjoints.

## Métrique de vérité (à remplir)

Tableau `prog · même_sortie · t_cpython · t_iron · speedup` + **speedup géométrique global**. Note « lib »
pour `sort` (délègue). Catalogue des **trous d'idiomes Iron** comblés / contournés.

## Trous de langage comblés (le cœur du test — Iron grandi par la batterie)

Écrire du Python débutant a révélé et fait combler ces idiomes dans `ironc` (vague 2 + en cours de fan-out) :

1. **`for c in s` → `char`** (était : pas d'itération de chaîne ; 1ʳᵉ tentative en `String` par caractère =
   allocation → Iron aussi lent que Python sur `wc`). Passage à `char` Rust : `wc` 1.02× → **27×**.
2. **`ord(c)` / `chr(n)`** (code ↔ caractère) — builtins absents.
3. **Méthodes chaîne** : `split()` sans arg (whitespace), `splitlines()`, `count(sub)`, **`sep.join(xs)`**
   (forme méthode ; n'existait qu'en free-fn `join(xs, sep)`).
4. **Slicing/indexation négatifs** `xs[-2:]`, `s[-1]`, `s[:-1]` (étaient : panic / index byte invalide).
5. **`//=`** (affectation floor-div) — non lexée (token `SlashSlashEq` ajouté).
6. **Répétition** `[x]*n`, `"ab"*n`, `n*[x]` — `*` sur list/str.
7. **`char` ↔ `str`** : comparaison `c == "x"` (char vs littéral 1-car → char), concat `s + c` / `c + c`
   (`format!`), inférence relâchée (pas d'unification forcée char/str).
8. **Rétro-propagation de type sur `xs.append(x)`** quand `xs = []` (élément inconnu) → `list[T]`.
9. **Bindings std** `args()` (`sys.argv`), `read_text()` (`open(p).read()`) — `import std` étoffé (pur Rust).

> Tous **sans annotation** : l'inférence V10/V11 tient. `gate-v11.sh` (langage) reste **VERT** après chaque ajout.

## Trous notés, NON comblés (contournés — dette future)

- **Identifiants = mots-clés Rust** (`mod`, `fn`, `let`, `type`…) non échappés (`r#mod`) → renommés dans les
  programmes (`mod`→`modulo`). À corriger dans `lower` (raw-ident).
- **`\xNN` / `\uNNNN`** échappements hexa non lexés → contournés (sentinelles via `ord`/booléens).
- **Indexation chaîne `s[i]` = O(n)** (`.chars().nth`) → en boucle chaude on convertit en `list[char]`
  d'abord (O(1)). Acceptable (idiome explicite).
- **`step` de slice** `s[::-1]` non supporté → reformulé (boucle / liste).
- **`sorted()`/`sort()`, `x in s`, `dict`** délèguent à la lib Rust → ces programmes (`sort`, `grep`, `uniq`,
  `wordfreq`, `anagram`) comparent des **librairies**, pas des langages → speedup ~1× **attendu et noté**.

## Métrique de vérité — perf CPython vs Iron→Rust (VERDICT)

**30/30** programmes : **compilent** (`ironc`, **0 `unsafe`**), **zéro annotation**, **même sortie** que
`python3` sur le dataset. Mono-cœur (`taskset -c 0`), min de N=3, mode résumé. **Speedup géométrique global :
14.73×.**

| Programme | t_CPython (s) | t_Iron→Rust (s) | speedup | profil |
|---|---:|---:|---:|---|
| seq | 1.73 | 0.008 | **227×** | boucle numérique pure |
| collatz | 3.03 | 0.037 | **82×** | numérique |
| dotproduct | 1.64 | 0.020 | **81×** | numérique |
| naivesort | 1.39 | 0.024 | **58×** | tri main (index O(1)) |
| od | 0.85 | 0.017 | **51×** | scan octets |
| editdist | 0.59 | 0.013 | **44×** | DP (char-list) |
| sieve | 2.91 | 0.079 | **37×** | crible |
| histogram | 0.64 | 0.020 | **31×** | scan octets |
| rle | 2.05 | 0.071 | **29×** | scan chars |
| tr | 4.00 | 0.143 | **28×** | translation chars |
| wc | 2.53 | 0.092 | **27×** | scan chars |
| cksum | 1.55 | 0.057 | **27×** | hash roulant |
| matmul | 0.17 | 0.006 | **27×** | numérique O(n³) |
| primecount | 1.35 | 0.060 | **22×** | numérique |
| jsonsum | 0.17 | 0.008 | **22×** | parse main |
| cut | 0.90 | 0.044 | **21×** | parse champs |
| csvsum | 0.88 | 0.043 | **20×** | parse champs |
| factor | 1.47 | 0.083 | **18×** | numérique |
| fold | 1.32 | 0.080 | **17×** | scan chars |
| nl | 1.01 | 0.063 | **16×** | scan chars |
| expand | 0.15 | 0.011 | **14×** | scan chars |
| tac | 3.28 | 0.281 | **12×** | scan chars |
| rev | 4.37 | 0.456 | **10×** | scan chars |
| gcdsum | 2.10 | 0.341 | **6×** | numérique (modulo lourd) |
| comm | 0.10 | 0.041 | **2.4×** | fusion (parse int) |
| wordfreq | 2.27 | 1.29 | **1.8×** | 🔸 dict (HashMap vs dict) |
| anagram | 0.12 | 0.10 | **1.2×** | 🔸 sort+dict (lib) |
| grep | 0.24 | 0.21 | **1.2×** | 🔸 sous-chaîne `in` (lib) |
| sort | 0.62 | 0.70 | **0.88×** | 🔸 `sorted()` (Timsort C vs Rust) |
| uniq | 0.11 | 0.14 | **0.78×** | 🔸 splitlines+compare (lib) |

🔸 **library-bound** (le travail est délégué à du C/Rust, pas au langage) → ~1× **attendu et honnête** :
`sorted()`, `dict`, sous-chaîne `in`, `splitlines`. Les programmes **langage-level** (scan de caractères,
boucles numériques) montrent le vrai écart interpréteur↔compilé : **10× à 230×**.

**Lecture honnête :** Iron→Rust écrase CPython sur le **calcul réel** (boucles, arithmétique, scan), d'un
ordre de grandeur typique **15–30×**, jusqu'à **230×** sur les boucles serrées. Là où Python délègue à ses
libs C (tri, dict, regex/substring), l'écart disparaît — c'est **exactement** ce qu'on attend, et ça valide
la métrique (on compare bien des **langages**, pas des librairies). Tout ça **sans une seule annotation de
type**, en réécrivant le `.py` quasi à l'identique (seuls imports/entrée rebindés).
