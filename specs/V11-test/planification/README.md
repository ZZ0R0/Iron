# V11-test — Planification (vagues / pistes)

> Phase 2. Découpe pour lancer **un max de pistes en parallèle, sans collision**. Fondation = contrat gelé
> (harnais + appariement `.py`/`.irn` + datasets) **et** prélude d'idiomes Iron. Puis fan-out des 30 programmes
> (ownership disjoint : un dossier par programme). Statut : 🔶 proposé — attend le feu vert **réalisation**.

## Carte des vagues

| Vague | Piste | Titre | Type | Parallèle ? |
|---|---|---|---|---|
| **1** | 1.1 | **Fondation** : harnais `bench.sh` + datasets + contrat + `factor` e2e | séquentiel | — |
| **2** | 2.1 | **Prélude idiomes Iron** : combler les idiomes débutant communs (ironc) | séquentiel | — |
| **3** | 3.x | **Fan-out bucket A** (16 coreutils compute) | parallèle | 16 pistes |
| **4** | 4.x | **Fan-out bucket B** (14 classiques compute) | parallèle | 14 pistes |
| **5** | 5.1 | **Unification + verdict** : tableau perf, speedup géométrique, catalogue des trous | séquentiel | — |
| **6** | 6.1 | **Bonus exotique (optionnel)** : threading multi-cœur (Rust vs GIL) | séquentiel | — |

> **Règle dure de parallélisme.** Les pistes 3.x/4.x **n'écrivent jamais dans `ironc/src`** (sinon collision).
> Elles écrivent **uniquement** dans leur dossier `tests/v11-test/<prog>/`. Tout **nouveau** trou d'idiome
> Iron rencontré en fan-out est **noté** (pas corrigé en parallèle) → corrigé en **séquentiel** (retour vague 2
> ou touch-up d'unification). Le langage (ironc) ne se modifie qu'en **vagues séquentielles**.

---

## Contrat gelé (fondation 1.1) — l'interface partagée

**Appariement.** Chaque programme = un dossier `tests/v11-test/<prog>/` contenant :
- `<prog>.py` — **vrai Python** débutant, **zéro type**, runnable `python3`. Termine par un appel `main()`.
- `<prog>.irn` — **même corps**, **zéro type**, entrée `def main()` (pas d'appel top-level), imports **rebindés**
  Iron. **Seules différences autorisées** : ligne d'entrée (`main()` vs `def main()`) + **imports/IO**
  (`sys.argv`→`args()`, `open(p).read()`→`read_file(p)`). La logique interne est **identique**.

**Invocation (gelée).** Un seul argument :
- programmes **texte** (lisent un dataset) : `<bin> <chemin_dataset>` → lisent, calculent, **impriment UNE ligne** (résumé).
- programmes **numériques** (génèrent en interne) : `<bin> <N>` (taille) → calculent, **impriment UNE ligne**.
- **Mode résumé obligatoire** : sortie = **une ligne** (compte / somme / hash). Zéro flux verbeux.

**Datasets (gelés).** `tests/v11-test/data/` généré **une fois** par `gen_data.py` (déterministe, sans
horodatage/aléa non semé) : ex. `text_big.txt` (~1–5 M lignes synthétiques), `nums_big.txt`, `csv_big.txt`,
`sorted_a.txt`/`sorted_b.txt` (pour `comm`). Tailles réglées pour **≥ 1 s côté CPython**.

**Sortie du bench (gelée).** `bench.sh` produit un tableau :
`programme · même_sortie(py3==iron) · t_cpython · t_iron · speedup`, + **speedup géométrique global**.
Protocole : `taskset -c 0`, N≥10 runs, **min** (+ médiane). Build Iron `--release`. Refuse un chrono si
sorties différentes.

---

## Les 30 programmes (chaque ligne = une piste ; template ci-dessous)

**Bucket A — coreutils à cœur de calcul *en langage*** (input = dataset texte ; sortie = résumé) :

| Piste | prog | cœur (boucle langage) | sortie résumé | input |
|---|---|---|---|---|
| 3.1 | `wc` | scan : compte lignes/mots/octets | `L W C` | text_big |
| 3.2 | `grep` | compte les lignes contenant une sous-chaîne fixe | `count` | text_big |
| 3.3 | `uniq` | replie doublons adjacents, `-c` | `nb_groupes` | text_sorted |
| 3.4 | `cut` | extrait champ N par délimiteur, par ligne | `somme_longueurs` | csv_big |
| 3.5 | `tr` | translation char→char (table), reconstruit | `hash_sortie` | text_big |
| 3.6 | `nl` | numérote lignes non vides | `derniere_num` | text_big |
| 3.7 | `fold` | wrap à largeur W | `nb_lignes_sorties` | text_big |
| 3.8 | `rev` | inverse les chars de chaque ligne | `hash_sortie` | text_big |
| 3.9 | `tac` | inverse l'ordre des lignes | `hash_sortie` | text_big |
| 3.10 | `comm` | fusion-compare 2 listes triées | `n1 n2 n3` | sorted_a/b |
| 3.11 | `seq` | génère 1..N, somme | `somme` | N |
| 3.12 | `od` | chaque octet → hex, concatène | `hash_sortie` | bin_big |
| 3.13 | `expand` | tabs → espaces (colonnes) | `nb_espaces` | text_tabs |
| 3.14 | `cksum` | hash roulant sur octets | `checksum` | bin_big |
| 3.15 | `factor` | division d'essai, somme du + grand facteur | `somme` | N (✅ PoC) |
| 3.16 | `sort` | tri lexical (⚠ délègue à `sorted()`/Rust-sort — **noté lib**) | `hash_1eres` | text_big |

**Bucket B — classiques compute qu'un débutant écrit** (génèrent/lisent ; sortie = résumé) :

| Piste | prog | cœur | sortie | input |
|---|---|---|---|---|
| 4.1 | `wordfreq` | dict de fréquences, max | `mot_top count` | text_big |
| 4.2 | `sieve` | crible d'Ératosthène ≤ N | `nb_premiers` | N |
| 4.3 | `histogram` | histogramme d'octets (256) | `octet_max` | bin_big |
| 4.4 | `naivesort` | **tri écrit à la main** (insertion/quick) | `hash_1eres` | nums_med |
| 4.5 | `csvsum` | somme d'une colonne CSV | `somme` | csv_big |
| 4.6 | `rle` | run-length encode | `len_encodé` | text_big |
| 4.7 | `jsonsum` | parse main d'une liste de nombres, somme | `somme` | json_big |
| 4.8 | `collatz` | plus longue chaîne de Collatz < N | `n_max long` | N |
| 4.9 | `editdist` | distance d'édition (DP) entre 2 chaînes | `dist` | 2 chaînes |
| 4.10 | `matmul` | produit matriciel dense N×N | `trace` | N |
| 4.11 | `gcdsum` | somme des gcd(i, N) pour i<N (Euclide) | `somme` | N |
| 4.12 | `primecount` | compte les premiers < N (test naïf) | `count` | N |
| 4.13 | `dotproduct` | produit scalaire répété de gros vecteurs | `somme` | N |
| 4.14 | `anagram` | groupes d'anagrammes (tri de clés) dans une liste | `nb_groupes` | words_big |

> **Template de piste (3.x / 4.x) — Definition of Done.**
> 1. `tests/v11-test/<prog>/<prog>.py` : Python débutant, zéro type, runnable `python3`.
> 2. `tests/v11-test/<prog>/<prog>.irn` : **même corps**, imports rebindés, `def main()`, zéro type.
> 3. **Même sortie** : `python3 <prog>.py <in>` == binaire Iron `<prog> <in>` (sur le dataset).
> 4. `.irn` **compile** (`ironc build`), **0 `unsafe`**, **0 annotation**.
> 5. Ligne de bench produite par `bench.sh` (t_py, t_iron, speedup).
> 6. Tout idiome Iron manquant rencontré → **noté** dans `realisation.md` (pas corrigé ici).
> **Fichiers possédés** : `tests/v11-test/<prog>/**` uniquement. **Aucune** modif `ironc/src`.

---

## Test mental de parallélisme (avant de lancer)

- Deux pistes 3.x/4.x touchent-elles le même fichier ? **Non** (dossiers disjoints).
- Attendent-elles une interface non figée ? **Non** : le contrat (invocation, datasets, format de sortie,
  appariement) est **gelé en 1.1**, et les idiomes communs en 2.1. → fan-out sûr.
- Risque résiduel : un programme a besoin d'un idiome Iron **non prévu** en 2.1 → **noté**, contourné ou
  différé ; correction ironc en **séquentiel**. Jamais deux agents dans `ironc/src` en parallèle.

---

## Vague 5 — unification + verdict

`bench.sh` agrège les 30 lignes → tableau perf + **speedup géométrique global** CPython→Iron. `realisation.md`
liste : N/30 compilent · N/30 même-sortie · trous d'idiomes comblés / contournés · note « lib » pour `sort`.
Verdict honnête (pas de moyenne gonflée par les cas triviaux).

## Vague 6 — bonus exotique (optionnel, « si ça me fait plaisir »)

Hors batterie mono-cœur. 1–2 programmes **multi-thread** (ex. somme parallèle / factorisation répartie) en
`.py` (threads → **bridés par le GIL**) vs `.irn` (`thread.spawn` → **vrai parallélisme** Rust). Montre que le
gain Iron **scale** avec les cœurs là où CPython plafonne. Marqué **bonus** — n'entre pas dans le speedup mono-cœur.
