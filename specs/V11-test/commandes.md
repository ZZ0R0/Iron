# V11-test — Les 30 commandes (contrat de périmètre)

> ⚠️ **Réorienté perf (cf. README §1-2).** La batterie est devenue un **benchmark CPython vs Iron→Rust**
> → on privilégie les commandes **compute-bound** (cœur de calcul en langage, mode résumé), bucket **A+B** du
> README. Les commandes **pur IO/fs** (cat/echo/head/tail/cp/mv/ls/find/du/basename/dirname/pwd…) passent en
> **bucket C — hors perf** (correction seulement, non chronométrées). Le tableau ci-dessous reste la référence
> du **cœur** de chaque commande ; la colonne « I/O » est indicative.

> Annexe du cadrage (`README.md`). Pour **chaque** commande : le **cœur** implémenté (« pas en entier »),
> le **hors-périmètre**, l'**I/O** (en idiomes **CPython** — cf. §3 du cadrage), et la commande coreutils de
> référence. Style imposé : **vrai Python débutant, zéro annotation**, fichier runnable sous `python3`.
>
> **Oracle réel = `python3 cmd.py`** (différentiel CPython vs Iron). La dernière colonne donne la commande
> **coreutils** servant d'**ancre** (sanity-check une fois que notre Python est correct).

Légende des tokens I/O des tableaux (en idiomes **CPython**) : `argv`=`sys.argv` · `file`=`open(p).read()` ·
`stdin`=`sys.stdin` · `out`=`print` · `write`=`open(p,"w")` · `fs`=`os`/`shutil` (remove/mkdir/rename/copy) ·
`dir`=`os.listdir`/`os.path` · `exit`=`sys.exit`.

---

## A. Texte / flux (18)

| # | Cmd | Cœur implémenté | Hors-périmètre | I/O | Oracle |
|---|-----|-----------------|----------------|-----|--------|
| 1 | `cat` | concatène fichiers (ou stdin si aucun) → stdout ; `-n` numérote | `-A -b -s -E` | argv·file·stdin·out | `cat` |
| 2 | `echo` | imprime les args joints par espace + `\n` ; `-n` sans newline | `-e` échappements | argv·out | `echo` |
| 3 | `wc` | compte lignes/mots/octets ; `-l -w -c` ; défaut = les 3 + nom | `-m -L`, multi-fichiers totaux | argv·file·stdin·out | `wc` |
| 4 | `head` | N premières lignes ; `-n N` (défaut 10) | `-c`, en-têtes multi-fichiers | argv·file·stdin·out | `head` |
| 5 | `tail` | N dernières lignes ; `-n N` (défaut 10) | `-f -c` | argv·file·stdin·out | `tail` |
| 6 | `grep` | lignes contenant une **chaîne fixe** ; `-i -v -c -n` ; exit 1 si 0 match | **regex**, `-r`, contexte | argv·file·stdin·out·exit | `grep -F` |
| 7 | `cut` | champs `-f` avec `-d`, **ou** caractères `-c` | plages complexes, `--complement` | argv·file·stdin·out | `cut` |
| 8 | `sort` | tri lexical ; `-r` inverse, `-n` numérique, `-u` unique | `-k -t`, stable | argv·file·stdin·out | `sort` |
| 9 | `uniq` | replie les doublons **adjacents** ; `-c -d -u` | `-f -s -i` | argv·file·stdin·out | `uniq` |
| 10 | `tr` | traduit SET1→SET2, **ou** `-d` supprime SET1 | `-s` squeeze, classes POSIX | argv·stdin·out | `tr` |
| 11 | `rev` | inverse les caractères de chaque ligne | — | file·stdin·out | `rev` |
| 12 | `tac` | imprime les lignes en ordre **inverse** | `-s` séparateur | file·stdin·out | `tac` |
| 13 | `nl` | numérote les lignes non vides | styles `-b`, `-w -s` | file·stdin·out | `nl` |
| 14 | `fold` | coupe chaque ligne à une largeur ; `-w` (défaut 80) | `-s` coupe aux espaces, `-b` | argv·file·stdin·out | `fold` |
| 15 | `paste` | fusionne les lignes correspondantes de fichiers (tab) | `-d -s` | argv·file·out | `paste` |
| 16 | `tee` | copie stdin → stdout **et** fichiers args | `-a` append | argv·stdin·write·out | `tee` |
| 17 | `comm` | compare 2 fichiers triés → 3 colonnes | `-1 -2 -3` suppression | argv·file·out | `comm` |
| 18 | `seq` | imprime FIRST..LAST (ou FIRST STEP LAST) | `-f` format, `-w` pad | argv·out | `seq` |

## B. Fichiers / système (12)

| # | Cmd | Cœur implémenté | Hors-périmètre | I/O | Oracle |
|---|-----|-----------------|----------------|-----|--------|
| 19 | `basename` | retire le répertoire ; retire le suffixe si fourni | `-a -z` multi | argv·out | `basename` |
| 20 | `dirname` | portion répertoire d'un chemin | `-z` | argv·out | `dirname` |
| 21 | `pwd` | imprime le répertoire courant | `-L/-P` | (cwd)·out | `pwd` |
| 22 | `ls` | entrées d'un dossier, **triées** ; `-a` (cachés), `-1` | `-l`, couleurs, `-R -t -S` | argv·dir·out | `ls -1` |
| 23 | `mkdir` | crée un dossier ; `-p` (parents) | `-m`, verbeux | argv·fs·exit | `mkdir` |
| 24 | `rmdir` | supprime un dossier **vide** | `-p`, verbeux | argv·fs·exit | `rmdir` |
| 25 | `rm` | supprime un fichier ; `-f` (ignore absent), `-r` (récursif) | `-i`, verbeux | argv·fs·exit | `rm` |
| 26 | `cp` | copie SRC DST (fichier) | `-r` dossiers, `-p -a` | argv·fs·exit | `cp` |
| 27 | `mv` | déplace/renomme SRC DST | `-i -n`, multi | argv·fs·exit | `mv` |
| 28 | `touch` | crée un fichier vide si absent | `-t -d`, mtime exact | argv·fs | `touch` |
| 29 | `find` | liste récursive sous DIR ; `-name SUBSTR` (fixe) | `-type` complexe, `-exec`, regex | argv·dir·out | `find … -name` |
| 30 | `du` | somme des tailles (octets) sous DIR ; `-s` résumé | blocs, `-h`, `--max-depth` | argv·dir·out | `du -b` |

---

## Notes de périmètre (transverses)

- **stdin vs fichier** : les filtres texte (cat/wc/head/tail/grep/cut/sort/uniq/tr/rev/tac/nl/fold) lisent
  **stdin** si aucun fichier n'est donné — comportement Unix standard, exercé par le harnais (pipe + fichier).
- **Codes de sortie** : `grep` (0/1), `mkdir/rmdir/rm/cp/mv` (0 succès, ≠0 erreur) via `exit_with`. Les
  filtres pures sortent 0.
- **Déterminisme** : `ls`/`find`/`du` → entrées curées + **tri** pour matcher l'oracle ; tailles `du` en
  **octets** (`du -b`) pour éviter les blocs dépendants du FS.
- **Idiomes Python d'un débutant** (vrai code, lu/écrit comme sous python3) :
  `import sys`, `data = sys.stdin.read()`, `for line in open(path)`, `with open(p,"w") as f:`,
  `parts = line.split(",")`, `words = line.split()`, `"\n".join(lines)`, `sys.argv[1:]`, `sys.exit(1)`,
  `import os; os.listdir(d)`, `os.path.basename(p)`. Chaque idiome refusé par Iron = **trou catalogué**.

## État vérifié des idiomes (spiké en cadrage — cf. README §2)

**Confirmés OK dans Iron** : `print` (dans un `def`), f-strings, `int()`/`str()`, `split(sep)`,
`sorted()`/`reversed()`, `enumerate()`, slicing **positif**.
**Confirmés trous** (à combler, README §3) : **statements au niveau module** (bloqueur n°1, **tous** les
fichiers), `import sys`/`os`, `open`/objets fichier, `with`, `sys.argv`/`stdin`/`exit`, `os.*` ; +
`split()` sans arg, **slice négatif** `xs[-n:]`, **`sep.join(xs)`**.
**Simplification v0 assumée** : lecture stdin/fichier = tout charger en mémoire (fichiers de test petits) —
`for line in f` itère une liste, pas un flux paresseux. OK pour la batterie.
