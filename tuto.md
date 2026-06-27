# Iron — mini-tuto par l'exemple

> **Iron** est un langage à **syntaxe Python** (indentation, `def`, `class`) et **sémantique par valeur**
> qui **transpile vers Rust** et compile via `cargo`. Pas de GC, pas de runtime lourd, et Iron **n'émet
> jamais d'`unsafe`** : le pire cas est une erreur `rustc` remontée sur ta ligne Iron.
>
> Deux principes :
> - **Inférence locale** : tu n'annotes **que ce qui n'est pas déductible**. La plupart des fonctions
>   s'écrivent sans aucune annotation, comme en Python.
> - **Zéro bibliothèque Iron** : les primitives du langage sont intégrées ; *tout le reste* (fichiers, JSON,
>   HTTP, regex, crypto…) se fait en **important un crate Rust**. L'écosystème d'Iron, c'est **crates.io**.

Fil rouge : on construit un petit **annuaire de contacts**, feature par feature. Chaque extrait est
exécutable tel quel — les sorties affichées sont réelles.

---

## 0. Le CLI `ironc`

```sh
ironc run    fichier.irn     # transpile → compile → exécute
ironc build  fichier.irn     # compile seulement (binaire)
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

def closest(query: str, names: list[str]):
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

`levenshtein` vient de la **crate Rust `strsim`**, compilée par `cargo`. Ici on annote `query: str` et
`names: list[str]` : `query` n'est consommé que par une fonction externe (Iron ne lit pas sa signature),
et le **type d'élément** d'une liste-paramètre n'est pas déductible localement. Le retour `best`, lui,
s'infère (`names[0]` est un `str`).

---

## 4. Des erreurs typées : `Result[T, E]`

Pour signaler un échec, on déclare un `enum` d'erreur et on renvoie `Result[T, E]`. On lève avec `raise`,
on récupère avec `.unwrap_or(...)` (ou `?` / `try`/`except`).

`validate.irn` :

```python
enum ValidationError:
    EmptyName
    TooYoung

def validate(name: str, age) -> Result[str, ValidationError]:
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

`age` est inféré (`age < 18`). On annote `name: str` (utilisé via `len`, ambigu sinon) et le retour
`Result[str, ValidationError]` : le **type d'erreur concret est explicite par choix** — c'est ce qui
permet à `?`/`try`/`except` de propager exactement cette erreur. (Sans annotation, `Result[T]` reste un
fourre-tout `IronError`.)

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

## 6. L'outillage du développeur

**Diagnostics (`ironc check`).** Iron s'appuie sur `rustc` comme filet : une erreur de type est remontée
**sur ta ligne Iron**.

```python
# bug.irn
def add(a, b):
    return a + b

def main():
    print(add(1, "deux"))   # str au lieu d'un nombre
```

```sh
$ ironc check bug.irn
ironc: erreur: bug.irn:5:5: error[E0308]: mismatched types
```

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

**VSCode.** L'extension (`editors/vscode/`) reconnaît les `.irn` (icône + coloration), branche le serveur de
langage (diagnostics live, complétion, hover, go-to-définition), et expose les commandes *Iron: Build / Run
/ Format / Add crate / Build (debug)*.

---

## 7. Quand faut-il annoter ?

Iron infère **localement** (à partir de l'usage dans le corps). Il n'y a **pas** d'inférence globale
Hindley-Milner : certaines annotations restent nécessaires. La règle pratique :

| Tu écris… | Annotation requise ? |
|---|---|
| arithmétique / comparaison à un nombre (`x * 2`, `age < 18`) | **non** — inféré `i64` |
| f-string / concaténation (`f"{x}"`, `s + "!"`) | **non** — inféré `str` |
| retour d'un calcul, d'une liste, d'une struct, d'un index | **non** — inféré |
| **champ** de `class` | **oui** (`name: str`) — comme un dataclass |
| **générique** | **oui** (`def f[T](x: T)`) |
| **erreur typée** `Result[T, E]` | **oui** — c'est l'opt-in qui rend `E` concret |
| **closure renvoyée** | **oui** (`-> fn(i64) -> i64`) |
| param passé **seulement** à une fonction externe / élément d'une liste-paramètre | **oui** (`query: str`, `names: list[str]`) |

En cas de doute : écris sans annotation, lance `ironc run`. Si Iron ne peut pas déduire, `rustc` te le dit
**sur ta ligne** — tu ajoutes l'annotation pile où il faut.

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
