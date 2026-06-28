# V11-test — Benchmark perf : CPython vs Iron→Rust sur programmes équivalents (cadrage)

> **Phase 1 — CADRAGE.** Contrat de la batterie **de performance**. Détail des programmes → `commandes.md`.
> Statut : 🔶 proposé — **PoC validée** (cf. §7), attend le feu vert pour la **planification**.

---

## 0. Objectif — mesurer le gain perf « même programme, CPython vs Iron→Rust »

On prend **~30 programmes** type commandes Linux, écrits **deux fois** :
- **`.py`** : vrai Python débutant, **zéro type**, qui tourne sous `python3` (la **référence** + la **baseline
  perf CPython**).
- **`.irn`** : **la même logique interne**, juste les imports/IO **adaptés aux bindings Iron** (pas de
  typedef, l'essentiel). Compilé en Rust natif par `ironc`.

On **benchmarke les deux** (mono-cœur, N exécutions) et on **compare CPython vs Iron→Rust**. Le test mesure
le **gain de perf du langage** (interpréteur Python vs Rust compilé) à logique **identique**.

> Ce reframe **supprime** le problème dur de « faire avaler du vrai Python à Iron » : la version Iron est
> écrite **en Iron** (`def main()`, `import std`, bindings), pas du CPython à parser. Les deux fichiers
> partagent le **corps**, diffèrent par l'**entrée** et l'**I/O**. **PoC à l'appui** (§7) : souvent le corps
> est **identique au caractère près**.

---

## 1. Règle d'or : **compute-bound**, pas IO-bound (sinon on mesure le noyau)

La plupart des coreutils sont **IO/syscall-bound** (cat/head/cp/ls…) : les benchmarker mesure le **kernel**,
pas le langage — **non pertinent**. Deux contraintes rendent la mesure **valide** :

1. **Mode silencieux / résumé.** Chaque programme lit son entrée **une fois**, fait le travail **en mémoire**,
   et n'imprime **qu'une ligne de résumé** (un compte, une somme, un hash). Zéro flux terminal → on mesure le
   **calcul**. (C'est ton « mode non verbeux ».)
2. **Boucle chaude en *langage*, pas en *librairie*.** Le travail doit être du code Python/Iron (boucles,
   arithmétique, scan de chaîne), **pas** un appel qui délègue tout à du C/Rust (`sorted()`, `hashlib`,
   `re`) — sinon on compare des **librairies**, pas des **langages**. Idiomes débutant OK tant que la **logique
   est dans le programme**.

---

## 2. Sélection adaptée des commandes (compute-bound, langage)

3 buckets. Le **benchmark perf** = bucket A (+ quelques B). Bucket C = **exclu du perf** (irréductiblement IO).

**A — coreutils à cœur de calcul *en langage*** (benchmarkés, gros input, mode résumé) :
`wc` (scan lignes/mots/octets) · `grep -c` (recherche sous-chaîne) · `uniq -c` (comptage adjacent) · `cut -f`
(split par ligne) · `tr` (translation char) · `nl` · `fold` (wrap) · `rev` · `tac` · `comm` (fusion triée) ·
`seq`+somme · `od` (octets→hex) · `expand` (tabs) · `cksum` (hash roulant) · `factor` (division d'essai) ·
`sort` *(⚠ délègue à Timsort/Rust-sort — comparé mais noté « librairie »)*. **(~16)**

**B — classiques compute qu'un débutant écrit** (compléter à ~30, fort en langage) :
fréquence de mots · crible de primes · histogramme d'octets · **tri écrit à la main** (vrai comparatif
*langage* du tri) · somme de colonne CSV · run-length encode · somme de nombres dans un JSON simple (parse
main) · plus longue chaîne de Collatz · distance d'édition (DP) · produit matriciel dense. **(~10)**

**C — pur IO/fs, EXCLU du perf** (gardés au plus comme tests de correction, **non chronométrés**) :
`cat echo head tail tee touch mkdir rmdir rm cp mv ls find du basename dirname pwd`.

> ✅ **Décidé** : la batterie perf = **~30 programmes compute-bound (A + B)**. Le **bucket C est exclu du
> perf** (au mieux correction, non chronométré) — les chronométrer mesurerait le noyau, pas le langage.

---

## 3. Les deux versions (contrat par programme)

| | `.py` (CPython) | `.irn` (Iron→Rust) |
|---|---|---|
| **Logique** | identique | identique (même algo, même boucle) |
| **Types** | aucun | aucun |
| **Entrée** | `sys.argv` / lecture fichier / génération | `args()` / `read_file()` / génération (bindings std) |
| **Sortie** | `print(résumé)` | `print(résumé)` |
| **Entrée prog.** | top-level + `main()` | `def main()` (entrée Iron) |

> Différence **minimale et mécanique** (imports/entrée). PoC `factor` : corps **identique**, seule la ligne
> d'appel `main()` en moins côté Iron. C'est ça « la même logique, juste l'essentiel adapté ».

---

## 4. Protocole de benchmark

- **Mono-cœur** : `taskset -c 0`. **N exécutions** (≥ 10), on garde le **min** (et la médiane) — le min élimine
  le bruit d'ordonnancement.
- **Workload dimensionné** : entrée fixe assez grosse pour que le **calcul domine le démarrage** (CPython
  ~20-30 ms de startup ; viser ≥ 1 s côté CPython). Datasets **générés une fois**, déterministes.
- **Mode résumé** : sortie = **une ligne** → IO négligeable.
- **Mesures** : par programme, `t_cpython`, `t_iron`, **speedup = t_cpython / t_iron**. Build Iron en
  `--release`.
- **Correction d'abord** : on n'accepte un chrono que si **sorties identiques** (`python3 x.py` == binaire Iron).
- **Sûreté** : Rust généré `#![forbid(unsafe_code)]`, **0 `unsafe`** (I2).
- **`tests/v11-test/bench.sh`** : agrège → tableau `programme · même_sortie · t_py · t_iron · speedup` + moyennes
  géométriques. (Optionnel : `hyperfine` si dispo, sinon boucle `taskset`+`date`.)

---

## 5. Critères d'acceptation (gate de la batterie)

1. **Correction** : pour chaque programme, **`.py` et `.irn` donnent la même sortie** sur tous les cas.
2. **Compile** : chaque `.irn` compile (0 `unsafe`), **zéro annotation** (logique débutant).
3. **Perf** : tableau **CPython vs Iron→Rust** par programme + **speedup géométrique global**. Honnête : si un
   programme délègue à une librairie (sort), c'est **noté** (compare des libs, pas des langages).
4. **Trous catalogués** : tout ce qu'Iron ne compile pas / oblige à adapter → noté (alimente le langage).

---

## 6. Hors-périmètre

- IO/fs-bound pur (bucket C) **hors perf**. Multi-thread (mono-cœur imposé). Régex (`re`). Gros nombres
  > i64 (bignum Python) — workloads choisis dans i64. Couverture exhaustive des options. Locale/encodage.

---

## 7. PoC — **validée** (preuve empirique, faite en cadrage)

`factor` (plus grand facteur premier par division d'essai, somme sur un intervalle — pur calcul langage) :

| Programme | Sortie py3 | Sortie Iron | Identique ? | CPython | Iron→Rust | Speedup |
|---|---|---|---|---|---|---|
| `factor` (×100 k) | 1149635731 | 1149635731 | ✅ | 0.436 s | 0.025 s | ~17× |
| `factor` (×500 k) | 23570426529 | 23570426529 | ✅ | (amorti) | (amorti) | **~21×** |

→ **Pipeline prouvé bout en bout** : même corps `.py`/`.irn`, même résultat, benchmarkable mono-cœur, gain
~20× sur du calcul pur. Le reste de la batterie = **répliquer ce schéma** sur ~30 programmes compute-bound.

---

## 8. Risques / questions ouvertes (🔶)

- ✅ **Baseline = CPython** (`python3`, interprété). Décidé : **pas de Cython** (comparaison « gain vs Python
  tel qu'on l'utilise »).
- 🔶 **Idiomes Iron** restants à combler côté `.irn` selon les programmes : `s.split()` sans arg, slice négatif,
  `sep.join`, `ord()`/`chr()`, `.count()` — chacun rencontré = trou comblé (mini, côté langage) ou contourné.
- 🔶 **Variance de mesure** : machine bruitée → min de N + médiane ; idéalement `taskset` + gouverneur perf.
- 🔶 **Bucket C** : exclu du perf (recommandé) ou adapté en use-case compute ? → ton arbitrage (§2).
- 🔶 **Représentativité** : factorisation/tri/scan = compute classique. Si tu veux un profil précis (ex.
  parsing lourd, numérique pur, manipulation de chaînes), je pondère la sélection.

---

## 9. Découpage pressenti (planif — **non décidé**)

1. **Fondation** : `bench.sh` (mono-cœur, N runs, résumé, compare py3↔iron) + générateur de datasets + 1
   programme e2e (`factor`, déjà prouvé) → colonne verte.
2. **Combler les idiomes Iron** rencontrés (split()/slice−/join/ord… au fil des programmes).
3. **Fan-out** : écrire les ~30 paires `.py`/`.irn` (bucket A puis B) — un dossier par programme, ownership disjoint.
4. **Unification + verdict** : `bench.sh` agrège le tableau perf + speedup géométrique + catalogue des trous.
