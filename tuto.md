# Iron — mini-tuto par l'exemple

> **Iron** est un langage à **syntaxe Python** (indentation, `def`, `class`) et **sémantique par valeur**
> qui **transpile vers Rust** et compile via `cargo`. Pas de GC, pas de runtime lourd, et Iron **n'émet
> jamais d'`unsafe`** : le pire cas est une erreur `rustc` remontée sur ta ligne Iron.
>
> Deux principes :
> - **Inférence pilotée par l'usage *et les appels*** : Iron compile le programme **entier**, donc il déduit
>   le type d'un paramètre de **la façon dont tu l'appelles**. Résultat : tu n'annotes **presque rien** — du
>   Python qui compile en Rust. (Si un type reste vraiment indéductible, Iron te le dit **sur ta ligne**.)
> - **Zéro bibliothèque Iron** : les primitives du langage sont intégrées ; *tout le reste* (fichiers, JSON,
>   HTTP, regex, crypto…) se fait en **important un crate Rust**. L'écosystème d'Iron, c'est **crates.io**.

Fil rouge : on construit un petit **annuaire de contacts**, feature par feature. Chaque extrait est
exécutable tel quel — les sorties affichées sont réelles.

---

## 0. Le CLI `ironc`

```sh
ironc run    fichier.irn     # transpile → compile → exécute
ironc build  fichier.irn     # compile seulement (binaire)
ironc emit   fichier.irn     # transpile → Rust LISIBLE (sans compiler), pour le reprendre à la main
ironc check  fichier.irn     # diagnostics riches (sans produire de binaire)
ironc fmt    fichier.irn     # formate en place
ironc repl                   # boucle interactive
ironc add    <crate>         # importe un crate Rust (génère le binding .iri)
ironc doc    fichier.irn     # documentation HTML
ironc new    mon_projet      # squelette de projet
```

Fichiers : **`.irn`** = source · **`.iri`** = binding (interface vers un crate Rust).

---

## 1. Les bases — sans annotation

`hello.irn` :

```python
def greet(name):
    return f"Bonjour, {name} !"

def main():
    print(greet("Iron"))
    squares = [n * n for n in range(1, 6)]   # compréhension de liste
    print(squares)
    print(len("café"))                       # len = nombre de caractères
```

```sh
$ ironc run hello.irn
Bonjour, Iron !
[1, 4, 9, 16, 25]
4
```

Aucune annotation : Iron déduit que `name` est une chaîne (utilisée dans une f-string) et que `greet`
renvoie une chaîne. `len` compte des **caractères** Unicode, pas des octets.

> Astuce : une fonction d'une ligne s'écrit `def carre(n): return n * n`.

---

## 2. Classes et sémantique par valeur

Une `class` regroupe des champs et des méthodes. **Tout est copié par valeur** : pas d'alias surprise,
pas de mutation à distance. Seuls les **champs** sont annotés (comme un `dataclass` Python).

`contact.irn` :

```python
class Contact:
    name: str
    age: i64

    def is_adult(self):
        return self.age >= 18           # comparaison → bool (inféré)

    def birthday(self):
        return Contact(name=self.name, age=self.age + 1)

def main():
    alice = Contact(name="Alice", age=17)
    older = alice.birthday()
    # `alice` n'a PAS changé : sémantique par valeur
    print(f"{alice.name}: {alice.age} (adulte? {alice.is_adult()})")
    print(f"{older.name}: {older.age} (adulte? {older.is_adult()})")
```

```sh
$ ironc run contact.irn
Alice: 17 (adulte? false)
Alice: 18 (adulte? true)
```

Les méthodes ne sont pas annotées : Iron déduit `is_adult → bool` et `birthday → Contact`. `birthday()`
renvoie un **nouveau** `Contact` ; l'original reste intact.

---

## 3. Le cœur d'Iron : importer un crate Rust

Besoin d'une recherche floue ? Pas de lib Iron — on **importe `strsim`** depuis crates.io. Une commande
génère le binding, puis on l'utilise comme du Iron natif.

```sh
$ ironc add strsim
`strsim` ajouté : strsim.iri écrit (10 fns/types bindés). `import strsim` est prêt.
```

`search.irn` :

```python
import strsim

def closest(query, names):
    best = names[0]
    for n in names:
        if levenshtein(query, n) < levenshtein(query, best):
            best = n
    return best

def main():
    names = ["Alice", "Bob", "Charlie", "Alicia"]
    print(closest("Alise", names))   # faute de frappe → plus proche
    print(closest("Bobby", names))
```

```sh
$ ironc run search.irn
Alice
Bob
```

`levenshtein` vient de la **crate Rust `strsim`**, compilée par `cargo`. **Zéro annotation** : Iron voit le
site d'appel `closest("Alise", names)` où `names` est une `list[str]` → il en déduit `query: str` et
`names: list[str]`, et le retour `best` (`names[0]` est un `str`). C'est l'inférence **pilotée par les
appels** : pas besoin de décrire les paramètres, Iron regarde comment tu t'en sers.

---

## 4. Des erreurs typées : `Result[T, E]`

Pour signaler un échec, on déclare un `enum` d'erreur et on renvoie `Result[T, E]`. On lève avec `raise`,
on récupère avec `.unwrap_or(...)` (ou `?` / `try`/`except`).

`validate.irn` :

```python
enum ValidationError:
    EmptyName
    TooYoung

def validate(name, age) -> Result[str, ValidationError]:
    if len(name) == 0:
        raise ValidationError.EmptyName
    if age < 18:                        # `age` inféré i64 (comparé à un nombre)
        raise ValidationError.TooYoung
    return ok(name)

def main():
    print(validate("Alice", 25).unwrap_or("refusé"))
    print(validate("", 25).unwrap_or("refusé"))
    print(validate("Bob", 15).unwrap_or("refusé"))
```

```sh
$ ironc run validate.irn
Alice
refusé
refusé
```

`name` et `age` sont inférés (appels `validate("Alice", 25)` + usages `len(name)`, `age < 18`). La **seule**
annotation est le retour `Result[str, ValidationError]` : déclarer le type d'erreur `E` est un **choix
explicite** — c'est l'opt-in qui permet à `?`/`try`/`except` de propager exactement cette erreur. (Sans lui,
`Result[T]` reste un fourre-tout `IronError`.)

---

## 5. Génériques, closures, parallélisme

`capstone.irn` :

```python
# Générique : `[T]` déclaré, `x: T`
def pair_of[T](x: T):
    return [x, x]

# Closure renvoyée → on annote le type fonction (sinon non déductible)
def multiplier(factor: i64) -> fn(i64) -> i64:
    return lambda x: x * factor

# Parallélisme — aucune annotation : tout s'infère
def range_sum(lo, hi):
    total = 0
    for i in range(lo, hi + 1):
        total = total + i
    return total

def parallel_sum(n):
    mid = n / 2
    h1 = thread.spawn(lambda: range_sum(1, mid))
    h2 = thread.spawn(lambda: range_sum(mid + 1, n))
    return h1.join() + h2.join()

def main():
    print(pair_of("hi"))         # ["hi", "hi"]
    triple = multiplier(3)
    print(triple(14))            # 42
    print(parallel_sum(100))     # 5050
```

```sh
$ ironc run capstone.irn
["hi", "hi"]
42
5050
```

`range_sum`/`parallel_sum` n'ont **aucune** annotation (accumulateur, `range`, division → tout numérique).
On annote là où l'inférence locale ne suffit pas : le paramètre d'un **générique** (`x: T`) et le **type
d'une closure renvoyée** (`fn(i64) -> i64`).

---

## 5b. Traits, opérateurs, motifs

Iron a de quoi modéliser proprement : **traits** (avec méthodes par défaut), **surcharge d'opérateurs**
(méthodes `__dunder__` → opérateurs Rust), et **motifs** riches (`match`).

`shapes.irn` :

```python
# Trait avec méthode par défaut
trait Shape:
    fn area(self) -> i64
    def describe(self):                       # méthode par défaut : réutilise area()
        return f"aire = {self.area()}"

class Square(Shape):
    side: i64
    def area(self):
        return self.side * self.side

# Surcharge d'opérateurs : `__add__` → l'opérateur `+`
class Vec2:
    x: i64
    y: i64
    def __add__(self, o):
        return Vec2(x=self.x + o.x, y=self.y + o.y)

# Motifs : alternatives `|`, intervalles `..`
def band(n):
    match n:
        0 | 1 | 2: return "bas"
        3 .. 10:   return "moyen"
        _:         return "haut"

def main():
    print(Square(side=5).describe())          # aire = 25
    v = Vec2(x=1, y=2) + Vec2(x=3, y=4)        # `+` surchargé
    print(f"({v.x}, {v.y})")                   # (4, 6)
    print(band(7))                             # moyen
```

```sh
$ ironc run shapes.irn
aire = 25
(4, 6)
moyen
```

Là encore, **presque rien n'est annoté** : seuls les **champs** (`side`, `x`, `y`) et la **signature
abstraite** du trait (`fn area(self) -> i64`) le sont. `describe`, `area` (impl), `__add__` infèrent leurs
types. L'ensemble béni des dunders : `__add__`/`__sub__`/`__mul__`/`__div__` (arithmétique), `__eq__`/`__lt__`
(comparaisons), `__str__` (affichage). Autres motifs utiles : `or`-patterns, intervalles, et `while/for … else`.

---

## 6. L'outillage du développeur

**Diagnostics (`ironc check`).** Iron a son **propre vérificateur de types** : il signale beaucoup d'erreurs
**lui-même, sur ta ligne Iron, avant `rustc`** (champ/méthode inconnus, mauvaise arité, conflit de types…).
`rustc` ne sert que de filet pour le reste.

```python
def add(a, b):
    return a + b

def main():
    print(add(1, "deux"))   # str, pas un nombre
```

```sh
$ ironc check bug.irn
ironc: erreur: bug.irn:5:11: type : `i64` attendu, trouvé `str`
```

Iron a inféré `a: i64` (premier appel) puis vu `"deux"` au second argument : conflit, signalé en clair.
C'est ce diagnostic que VSCode souligne **en direct**.

**REPL (`ironc repl`).** Pour explorer :

```sh
$ ironc repl
>>> x = 21
>>> x * 2
42
>>> def sq(n): return n * n
>>> sq(x)
441
```

**Formatage (`ironc fmt`).** Réindente en 4 espaces, préserve les commentaires, idempotent :

```sh
$ ironc fmt messy.irn      # indentation à 6 espaces → 4
formaté : messy.irn
$ ironc fmt messy.irn      # relancer ne change plus rien
déjà formaté : messy.irn
```

**Doc & debug.** `ironc doc fichier.irn` génère la doc HTML (via rustdoc). `ironc build --debug fichier.irn`
produit un binaire avec infos de debug **plus** une `sourcemap.json` qui relie les lignes Rust ↔ Iron — tu
débogues avec gdb/lldb et tu retombes sur tes lignes `.irn`.

**Émission du Rust (`ironc emit`).** À la Nim (`nimcache`) : Iron peut produire un **Rust lisible** —
sans marqueurs de source-map — dans un dossier visible (un projet cargo complet), **sans compiler**.

```sh
$ ironc emit factor.irn
Rust propre : factor-rust/src/main.rs
  → édite-le, puis : cd factor-rust && cargo run
```

Tu obtiens du Rust **reprenable à la main** : noms préservés, structure 1:1, types clairs. Pratique pour
**fignoler les perfs**, ou **interfacer haut/bas niveau** (tu écris un driver/algorithme en Rust, tu
l'`import`es comme crate côté Iron, et tu vois le tout dans le `.rs` final). Dans un projet, le manifeste
`ironc.toml` pilote ça :

```toml
[iron]
entry    = "src/main.irn"
emit     = "clean"     # "clean" (lisible) | "instrumented" (avec source-maps)
rust-out = "rust"      # où ironc emit écrit le Rust
```

`ironc build`/`run` restent **instrumentés** (les erreurs `rustc` remontent sur ta ligne Iron) ; `ironc emit`
est le mode « donne-moi le Rust propre ». Les `.clone()` restants signalent justement les points chauds à
optimiser à la main (cf. `--audit`).

**VSCode.** L'extension (`editors/vscode/`) reconnaît les `.irn` (icône + coloration), branche le serveur de
langage (`ironc lsp` → diagnostics **live**, complétion, hover), souligne les erreurs **à la sauvegarde**
(`ironc check`), et expose les commandes *Iron: Build / Run / Format / Add crate / Debug* + un **CodeLens
▶ Run** au-dessus de `def main()`. Mise en route :

```sh
cargo build --release                          # construit `ironc` (puis mets-le sur ton PATH)
cd editors/vscode && npm install && npm run compile   # compile l'extension
# puis F5 (Extension Development Host) ou `npx vsce package` → installe le .vsix
```

---

## 7. Quand faut-il annoter ?

Iron infère depuis **l'usage dans le corps** *et* **les sites d'appel** (il compile tout le programme). Il
n'y a **pas** d'inférence globale Hindley-Milner, mais en pratique tu n'annotes presque rien. La règle :

| Tu écris… | Annotation requise ? |
|---|---|
| arithmétique / comparaison à un nombre (`x * 2`, `age < 18`) | **non** — inféré `i64` |
| f-string / concaténation (`f"{x}"`, `s + "!"`) | **non** — inféré `str` |
| retour d'un calcul, d'une liste, d'une struct, d'un index | **non** — inféré |
| **paramètre** (même passé à une fonction externe, même `list[str]`) | **non** — inféré du **site d'appel** |
| param appelé avec des **types incompatibles** à deux endroits | **non** — devient **générique** automatiquement |
| **champ** de `class` | **oui** (`name: str`) — comme un dataclass |
| **générique déclaré** | **oui** (`def f[T](x: T)`) |
| **erreur typée** `Result[T, E]` | **oui** — l'opt-in qui rend `E` concret |
| **closure renvoyée** | **oui** (`-> fn(i64) -> i64`) |

> En clair : il ne reste **que 4 endroits** où tu annotes — **champs de classe**, **génériques déclarés
> `[T]`**, **erreurs typées `Result[T, E]`**, **type d'une closure renvoyée**. Tout le reste (params,
> retours, locaux) s'infère. En cas de doute : écris sans annotation, lance `ironc run` ; si un type reste
> indéductible, Iron te le dit **sur ta ligne**.

---

## 8. Récap

| Tu veux… | En Iron |
|---|---|
| une fonction | `def f(x):` (annote si non déductible) |
| un type produit | `class C:` (champs typés, sémantique par valeur) |
| un type somme | `enum E:` |
| une bibliothèque | `ironc add <crate>` puis `import <crate>` — **jamais** de lib Iron |
| gérer l'échec | `Result[T, E]` + `raise` / `?` / `try`/`except` |
| du générique | `def f[T](x: T)` |
| une fonction-valeur | `lambda x: ...`, type `fn(A) -> R` |
| du parallélisme | `thread.spawn(lambda: ...)` / `.join()` |

**Le réflexe Iron** : tu écris en syntaxe Python *avec un minimum d'annotations*, tu penses en valeurs, et
dès qu'il te faut une lib, tu `import` un crate Rust. Iron génère du Rust sûr, `cargo` le compile.

Pour aller plus loin : `tests/vitrine-v9.1/` est un projet multi-fichiers qui combine tout ce tuto.
