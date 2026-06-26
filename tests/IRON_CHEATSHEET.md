# Iron cheatsheet (pour écrire un clone fidèle d'un programme Python)

Iron = **syntaxe Python, compile vers Rust**. Tu écris un `.py` (oracle), puis le **même** programme en
`.irn`, tu compiles, et la sortie doit être **identique au caractère près**.

## Lancer

- Python : `python3 chemin/main.py`
- Iron : `cargo run --quiet --manifest-path /home/guts/Documents/Iron/ironc/Cargo.toml -- run chemin/main.irn`
- Comparer : `diff <(python3 .../main.py) <(cargo run ... -- run .../main.irn)` → **vide** = OK.

## Multi-fichiers

`import nom` charge `nom.irn` (même dossier), items accessibles **à plat** (`import primes` → `sieve(...)`).
Le `.py` miroir utilise `from primes import sieve, ...`.

## Syntaxe (identique à Python sauf annotations)

```
def f(x: i64, y: i64 = 10) -> i64:      # types annotés ; défaut OK
    return x + y

class Point:                             # class = champs + méthodes
    x: i64
    y: i64
    def dist2(self) -> i64:
        return self.x * self.x + self.y * self.y
    def move(self, dx: i64):             # self mutant OK (self.x = ... ou self.list.append(...))
        self.x = self.x + dx

p = Point(x=3, y=4)                       # construction par mots-clés (champs)
p.move(2)                                 # appel méthode positionnel
```

**Supporté** : `if/elif/else`, `for x in xs`, `for i, v in enumerate(xs)`, `while`, `break`/`continue`,
`match`/`case` (via `match s:` + motifs), f-strings `f"{x}"`, compréhensions `[e for x in xs if c]`
(liste/dict/set), tuples `a, b = f()` et `t.0`, slicing `xs[1:3]`, ternaire `a if c else b`, `in`/`not in`,
`assert`, `try/except/raise`, `and/or/not`, `True/False/None`, `'…'` et `"…"`, concat `s1 + s2`,
`+= -= *= ** // %`, `enum`/`match`.

**Stdlib** : `print, len, range, abs, min, max, sum, sorted, reversed, enumerate, zip, map, filter, reduce`,
`int(s), float(s), str(x), round(x), join(xs, sep)`. Méthodes : `s.upper/lower/strip/split/replace/
startswith/endswith/contains`, `xs.append/pop/sort/reverse/contains/len`, `d.keys/values/items/get/contains`.
Fichiers : `io.read_file/write_file/read_lines` (Result, via `?` dans try). `set`, `ref()`/`shared` (Rc).

## Types

`i64` (entier par défaut), `f64`, `bool`, `str`, `char`, `list[T]`, `dict[K, V]`, `set[T]`, `(A, B)` tuple,
`Option[T]`, `Result[T]`. Param non annoté → inféré. Annoter les retours aide.

## ⚠️ LIMITES (workaround obligatoire — sinon erreur)

1. **Args nommés** : seulement sur construction `Point(x=…)` et fn tout-nommé `f(a=…, b=…)`. PAS sur
   méthode (`p.move(dx=2)` ❌ → `p.move(2)`), PAS en mixte (`sorted(xs, key=f)` ❌). → **positionnel**.
2. **Pas de `return <valeur>` dans `try`** → assigner une variable-résultat, return après le try.
3. **HOF** : param fonction supposé `i64→i64`. Autres signatures → lambda inline / map / filter.
4. **Collection vide** : `d = {}` puis `d[k] = v` ✅ (inféré). `xs = []` puis `xs.append(x)` ✅.
5. Pas d'héritage de classe, pas de `yield`/générateurs, pas de décorateurs, `*args`/`**kwargs`,
   pas de regex/JSON, pas de `with`. Pas de format-spec `f"{x:.2f}"`.

## ⚠️ RÈGLES DE SORTIE (sinon le diff échoue à tort — Python ≠ Rust formatage)

- ✅ Imprimer : **entiers**, **chaînes explicites**, **listes/tuples d'entiers** (`[1, 2, 3]` identique).
- ❌ NE PAS imprimer en brut : **flottants** (Python `1.0` ≠ Rust `1`), **booléens** (`True` ≠ `true`),
  **listes de chaînes** (`['a']` ≠ `["a"]`), **dicts** (ordre+quotes). À la place :
  - booléen → `"yes" if b else "no"` (ou `1`/`0`).
  - flottant → travailler en **entiers** (centimes : `int(x * 100)`), ou arrondir `round(x)`.
  - dict / collection → **trier puis itérer** : `for k in sorted(d.keys()): print(f"{k}={d[k]}")`.
  - liste de chaînes → `print(join(xs, ","))` (identique des deux côtés).
- **Déterminisme** : aucun `random`, aucun `time`. Le `.py` et le `.irn` doivent suivre la MÊME logique.

## Exemple de référence qui marche

Voir `tests/projects/23_number_theory/` (python/ + iron/) : multi-fichiers, sortie entière/listes,
diff identique. **Reproduis exactement ce style.**
