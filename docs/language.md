# Le langage Iron

Surface **Python**, sémantique **par valeur**, typage **inféré**. Tout exemple ci-dessous est du Iron valide.
Pour la sémantique mémoire (clone/move/emprunt) voir [semantics.md](semantics.md) ; pour les pièges,
[gotchas.md](gotchas.md).

## Indentation & commentaires
Blocs par **indentation** (comme Python). Commentaires `#`. Pas de `;` ni d'accolades.

## Constantes & variables
```python
MAX = 100                # constante globale (convention MAJUSCULE) → const Rust
GREETING = "salut"

def main():
    a = b = 5            # affectation chaînée
    x, y = 1, 2          # déstructuration
    a = a + 1            # réassignation locale
```

## Fonctions
```python
def double(x):                 # types inférés depuis l'appel
    return x + x

def greet(name: str) -> str:   # annotations optionnelles
    return "salut " + name

def outer(x):
    def inner(y):              # fonction imbriquée (closure)
        return y + y
    return inner(x) + 1

def add(a, b=10):              # valeur par défaut (surface Python)
    return a + b

def announce[T: Show](x: T):   # générique BORNÉ (déclaration explicite [T: Bound])
    print(x.shout())
```
- **L'inférence est pilotée par les sites d'appel** : le type d'un paramètre = ce avec quoi on l'appelle.
  En pratique **zéro annotation** ; restent explicites : la **déclaration** d'un générique `[T]` et le **type
  d'une closure renvoyée**.
- `async def` existe (runtime tokio injecté) mais l'async exotique est hors-périmètre.

## Types
| Iron | Rust | Note |
|---|---|---|
| `i64`, `f64` | `i64`, `f64` | défauts numériques |
| `i8 i16 i32 i128 isize u8 u16 u32 u64 u128 usize f32` | idem | zoo numérique (annoter ou caster `to_u32(x)`) |
| `bool`, `char`, `str`, `unit` | `bool`, `char`, `String`, `()` | `str` = `String` possédée |
| `list[T]` | `Vec<T>` | |
| `dict[K, V]` | `HashMap<K, V>` | |
| `set[T]` | `HashSet<T>` | |
| `(A, B, …)` | tuple | |
| `Option[T]` | `Option<T>` | inféré depuis un chemin `None` |
| `Result[T]` / `Result[T, E]` | `Result<T, IronError>` / `Result<T, E>` | voir Erreurs |
| `shared[T]` | `Rc<RefCell<T>>` | partage mutable explicite (I5) |
| `Name` / `Name[T]` | struct/enum Iron ou type externe | |

## Collections & littéraux
```python
xs = [1, 2, 3]                 # list[i64]
d  = {"a": 1, "b": 2}          # dict[str, i64]
s  = {1, 2, 3}                 # set[i64]
t  = (1, "x", 3.0)             # tuple
xs.append(4); print(len(xs)); print(xs[0])
print(xs[1:3])                 # slicing
```

## Flux de contrôle
```python
if x > 0:
    print("positif")
elif x == 0:
    print("zéro")
else:
    print("négatif")

for v in xs:
    if v < 0:
        break
    continue
else:                          # for/while-else (exécuté si pas de break)
    print("aucun négatif")

while not done:
    done = step()

label = "pair" if n % 2 == 0 else "impair"     # ternaire
```

## Compréhensions & f-strings
```python
carres = [x * x for x in range(5)]
pairs  = [x for x in xs if x % 2 == 0]
table  = {k: v for k, v in items}              # dict-comprehension
uniq   = {x for x in xs}                       # set-comprehension
msg    = f"{name} a {len(xs)} éléments"        # f-string
```

## Lambdas & closures de 1re classe
```python
add = lambda a, b: a + b           # corps mono-expression ; types inférés par rustc
xs.sort_by_key(lambda e: e.score)

def apply(f, x):                   # ordre supérieur : `f` générique borné `Fn` (coût nul, monomorphisé)
    return f(x)
print(apply(lambda n: n * 2, 21))  # → 42

def adder(n):                      # closure RETOURNÉE (capture `n` par valeur)
    return lambda x: x + n
print(adder(10)(5))                # → 15  (appel chaîné)

print((lambda x: x * x)(7))        # IIFE → 49

struct Op:                         # closure STOCKÉE en champ → `Box<dyn Fn>`
    f: fn(i64) -> i64
op = Op(f=lambda x: x * x)
print(op.f(6))                     # → 36  (appel du champ-closure, pas un dispatch de méthode)
```
Une closure passée à une fn utilisateur est **monomorphisée** (`impl Fn`, zéro coût) ; stockée en champ/liste
elle devient `Box<dyn Fn>`. Limite : une **liste** de closures **capturantes** hétérogènes n'est pas émise
(→ champ de struct, ou lambdas non-capturantes). Voir [gotchas #14](gotchas.md).

## Concurrence : async/await & thread.scope
```python
async def fetch(x: i64) -> i64:
    return x * 2
async def main():
    print(await fetch(21))         # → 42  (runtime tokio injecté SEULEMENT si `async` présent, I5)
```
```python
def main():
    data = [10, 20, 30, 40]        # partagé en LECTURE : emprunté `&data`, PAS cloné par worker
    total = ref_sync(0)            # état mutable partagé thread-safe (Arc<Mutex>)
    thread.scope(lambda s: [s.spawn(lambda: lock_add(total, data[i])) for i in range(4)])
    print(lock_get(total))         # → 100  (les workers rejoignent en fin de scope)
```
`thread.scope` ouvre un scope dont les workers `s.spawn(…)` **empruntent** les données partagées (bénéfice
ownership : lecture `&scene` sans clone). Corps = mono-expression ; orchestration riche → `rust { … }`.

## match (V11)
```python
def grade(n):
    match n:
        0 | 1 | 2:             # or-pattern
            return "bas"
        3..=5:                 # range inclusif (.. = exclusif)
            return "moyen"
        _:                     # joker
            return "haut"
```

## Structs, enums, traits, classes
**`class`** = struct + (optionnellement) implémentation d'un trait, façon Python :
```python
class Vec2:                    # struct simple
    x: f64
    y: f64
    def length(self) -> f64:
        return sqrt(self.x * self.x + self.y * self.y)

p = Vec2(x=3.0, y=4.0)         # construction par mots-clés
print(p.length())             # appel de méthode
```
```python
trait Show:
    fn show(self) -> str       # signature
    fn shout(self) -> str:     # méthode par défaut (avec corps)
        return self.show() + "!"

class Dog(Show):               # class qui IMPLÉMENTE un trait
    name: str
    def show(self) -> str:
        return self.name
```
```python
enum Shape:                    # enum à variantes (tuple)
    Circle(i64)
    Rect(i64, i64)

def area(s):                   # `s` inféré Shape (le match nomme ses variantes)
    match s:
        Shape.Circle(r):
            return r * r * 3
        Shape.Rect(w, h):
            return w * h

print(area(Shape.Circle(2)))   # construction : Enum.Variant(args)
```
> Il existe aussi le mot-clé `struct Name:` (struct pure). En pratique `class` couvre struct **et** OOP.

## Erreurs : Option, Result, `?`, try/except
```python
def parse(s) -> Result[i64]:
    v = to_int(s)?             # `?` propage l'erreur (sucre) — voir le PIÈGE ci-dessous
    return v

def main():
    try:
        n = parse("42")?
        print(n)
    except:                    # capture les erreurs Result/? du corps
        print("invalide")
    raise SomeError("boom")    # → return Err(IronError(...))
```
- Un chemin qui renvoie `None` → `Option[T]`. Une opération faillible → `Result[T, IronError]`, propagée par `?`.
- **`NaN` n'est pas `None`** : un `f64` NaN reste un `f64` valide.

> ⚠️ **PIÈGE majeur** : `?` n'est utilisable que dans une fonction qui **rend `Result`/`Option`**, et **`main`
> ne se promeut PAS** automatiquement. Pattern : mettre la logique dans `def run() -> Result[T]` et l'appeler
> depuis `main` via `try/except`. Détails et autres pièges → [gotchas.md](gotchas.md).

## Opérateurs
Arith `+ - * / % ** //` · comparaison `== != < <= > >=` · logique `and or not` · appartenance `in` / `not in`
· bit-à-bit `& | ^ << >>`. **Sur les types externes**, préférer les **méthodes** aux opérateurs (voir
[gotchas.md](gotchas.md)).

## Interop inline
```python
rust {
    // Rust verbatim (interop niveau 1) — échappatoire pour ce qu'Iron n'exprime pas.
    let x = 42;
}
```
Décorateurs d'attribut Rust : `@derive(serde::Serialize)`, `@repr(C)` au-dessus d'un item.
