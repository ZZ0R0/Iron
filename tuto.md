# Iron — mini-tuto par l'exemple

> **Iron** est un langage à **syntaxe Python** (indentation, `def`, `class`) et **sémantique par valeur**
> qui **transpile vers Rust** et compile via `cargo`. Pas de GC, pas de runtime lourd, et Iron **n'émet
> jamais d'`unsafe`** : le pire cas est une erreur `rustc` remontée sur ta ligne Iron.
>
> Principe directeur : **zéro bibliothèque Iron**. Les primitives du langage sont intégrées ; *tout le
> reste* (fichiers, JSON, HTTP, regex, crypto…) se fait en **important un crate Rust**. L'écosystème
> d'Iron, c'est **crates.io**.

Fil rouge de ce tuto : on construit un petit **annuaire de contacts**, feature par feature. Chaque extrait
est exécutable tel quel — les sorties affichées sont réelles.

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

## 1. Les bases

`hello.irn` :

```python
def greet(name: str) -> str:
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

À retenir : indentation significative, annotations de type `name: str`, f-strings `f"...{x}..."`,
compréhensions, et `len` qui compte des **caractères** Unicode (pas des octets).

> Astuce : une fonction d'une ligne s'écrit `def carre(n: i64) -> i64: return n * n`.

---

## 2. Classes et sémantique par valeur

Une `class` regroupe des champs et des méthodes. **Tout est copié par valeur** : pas d'alias surprise,
pas de mutation à distance.

`contact.irn` :

```python
class Contact:
    name: str
    age: i64

    def is_adult(self) -> bool:
        return self.age >= 18

    def birthday(self) -> Contact:
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

`birthday()` renvoie un **nouveau** `Contact` ; l'original reste intact. (Sous le capot, Iron emprunte
quand c'est sûr et ne clone que si nécessaire — sans jamais d'`unsafe`.)

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

def closest(query: str, names: list[str]) -> str:
    best = names[0]
    best_d = levenshtein(query, best)
    for n in names:
        d = levenshtein(query, n)
        if d < best_d:
            best_d = d
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

`levenshtein` vient de la **crate Rust `strsim`**, compilée par `cargo`. Iron a juste généré l'appel —
zéro code de bibliothèque Iron. Même `import std` (fichiers, env) marche de cette façon.

---

## 4. Des erreurs typées : `Result[T, E]`

Pour valider une entrée, on déclare un `enum` d'erreur et on renvoie `Result[T, E]`. On lève avec `raise`,
on propage avec `?`, on récupère avec `try`/`except`.

`validate.irn` :

```python
enum ValidationError:
    EmptyName
    TooYoung

def validate(name: str, age: i64) -> Result[str, ValidationError]:
    if len(name) == 0:
        raise ValidationError.EmptyName
    if age < 18:
        raise ValidationError.TooYoung
    return ok(name)

def describe(name: str, age: i64) -> str:
    try:
        v = validate(name, age)?        # `?` propage l'erreur typée
        return f"OK: {v}"
    except e:                           # `e` est de type ValidationError
        match e:
            ValidationError.EmptyName:
                return "refusé: nom vide"
            ValidationError.TooYoung:
                return "refusé: mineur"

def main():
    print(describe("Alice", 25))
    print(describe("", 25))
    print(describe("Bob", 15))
```

```sh
$ ironc run validate.irn
OK: Alice
refusé: nom vide
refusé: mineur
```

`Result[str, ValidationError]` porte un type d'erreur **concret** ; `?` et `try/except` le propagent.
(Sans annotation, `Result[T]` reste un fourre-tout `IronError` — l'erreur typée est un opt-in.)

---

## 5. Génériques, closures, parallélisme

`capstone.irn` :

```python
# Générique utilisateur : marche pour n'importe quel T
def pair_of[T](x: T) -> list[T]:
    return [x, x]

# Closure renvoyée puis stockée
def multiplier(factor: i64) -> fn(i64) -> i64:
    return lambda x: x * factor

# Parallélisme natif : somme 1..n sur deux threads
def range_sum(lo: i64, hi: i64) -> i64:
    total = 0
    for i in range(lo, hi + 1):
        total = total + i
    return total

def parallel_sum(n: i64) -> i64:
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

Génériques (`[T]`), closures de 1ʳᵉ classe (`fn(i64) -> i64`, `lambda`) et threads (`thread.spawn`/
`.join()`) — le tout transpilé en Rust idiomatique, **0 `unsafe`**.

---

## 6. L'outillage du développeur

**Diagnostics (`ironc check`).** Iron s'appuie sur `rustc` comme filet : une erreur de type est remontée
**sur ta ligne Iron**.

```python
# bug.irn
def add(a: i64, b: i64) -> i64:
    return a + b

def main():
    print(add(1, "deux"))   # str au lieu de i64
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
>>> def sq(n: i64) -> i64: return n * n
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

## 7. Récap

| Tu veux… | En Iron |
|---|---|
| une fonction | `def f(x: i64) -> i64:` |
| un type produit | `class C:` (sémantique par valeur) |
| un type somme | `enum E:` |
| une bibliothèque | `ironc add <crate>` puis `import <crate>` — **jamais** de lib Iron |
| gérer l'échec | `Result[T, E]` + `raise` / `?` / `try`/`except` |
| du générique | `def f[T](...)`, `class Box[T]:` |
| une fonction-valeur | `lambda x: ...`, type `fn(A) -> R` |
| du parallélisme | `thread.spawn(lambda: ...)` / `.join()` |

**Le réflexe Iron** : tu écris en syntaxe Python, tu penses en valeurs, et dès qu'il te faut une lib, tu
`import` un crate Rust. Iron génère du Rust sûr, `cargo` le compile.

Pour aller plus loin : `tests/vitrine-v9.1/` est un projet multi-fichiers qui combine tout ce tuto en un
seul programme.
