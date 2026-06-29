# ⚠️ Pièges & subtilités

Les choses qu'on apprend **dans la douleur** en écrivant du Iron. À lire avant d'écrire du Iron sérieux —
et **surtout** si tu es un agent qui reprend le projet : ceci t'évite de re-déduire ces règles depuis le source.

---

## 1. `?` — `main` (et toute fn non annotée) est **auto-promu** `Result` *(V18 — levé)*

```python
def main():
    v = parse("42")?          # ✅ V18 : `main` devient `-> Result<(), IronError>`, chute = `Ok(())`
    print(v)
```
Une fn **sans annotation** qui utilise `?` (hors `try/except`) est promue `Result` par le flux (I11) — `main`
compris. Le `?` propage proprement (plus de E0277 rustc brut). Le pattern helper+`try/except` reste valide
si tu veux **gérer** l'erreur localement plutôt que la propager :
```python
def main():
    try:
        print(parse("42")?)
    except:
        print("erreur")
```

## 2. N'annote pas `Result[unit]` à la main

```python
def main() -> Result[unit]:    # ❌ émet `Result<unit, …>` — `unit` n'est pas un type Rust → E0425
```
Laisse `main` **sans annotation** et utilise le pattern helper+try/except ci-dessus.

## 3. Ascription : `x: T = value` pour **figer** des génériques / const

L'inférence ne peut pas deviner un paramètre const ou un générique non contraint. **Annote la variable** :
```python
v: ArrayVec[i64, 8] = ArrayVec.new()   # fige la capacité const 8 → arrayvec::ArrayVec::<i64, 8>
cfg: Config = from_str(text)?          # fige le type → rustc infère from_str::<Config>
```
L'ascription émet `let v: T = …` ; `rustc` infère le reste. Indispensable pour les const-génériques
(arrayvec, fixed…) et la désérialisation serde typée.

## 4. Sur les **types externes/valeur**, préfère les MÉTHODES aux opérateurs

C'est **la** subtilité des types math (vecteurs, matrices, Decimal…). Deux raisons :
1. Iron n'a **qu'un dunder par opérateur** (Python surcharge `*` pour scalaire ET terme-à-terme ; pas Iron).
2. Un opérateur surchargé prend `self` **par valeur**, mais l'analyse d'échappement passe souvent ces valeurs
   **par référence** → `&Vec3 + Vec3` ne compile pas. Les **méthodes** (`&self`, autoref) marchent sur les
   valeurs empruntées.

```python
# ❌ fragile :        c = a + b
# ✅ robuste :        c = a.add(b)         /  v.scale(s)  /  v.dot(o)  /  v.neg()
```
(Les opérateurs marchent quand même sur des types *possédés* simples comme `Decimal`/`Ratio` — mais en cas de
doute sur un type externe, la méthode est le choix sûr.)

## 5. `.copy()` pour éviter un **move prématuré**

Quand un même objet possédé sert **deux fois** dans une expression et qu'un appel le **consomme** :
```python
return v.sub(n.scale(2.0 * v.dot(n.copy())))   # n.copy() : sinon n est "déplacé" trop tôt
```

## 6. `return` conditionnel dans une boucle → inférence `Option`

```python
def pick() -> Vec3:
    while True:
        p = random_vec()
        if p.ok():
            return p           # ⚠️ un seul return conditionnel dans une boucle → infère Option[Vec3]
```
Iron infère `Option` parce que le flux peut « ne pas retourner ». **Fix** : boucler sur un drapeau pour avoir
**un `return` final certain** :
```python
def pick() -> Vec3:
    found = False
    p = zero()
    while not found:
        p = random_vec()
        if p.ok():
            found = True
    return p                   # ✅ return certain → type Vec3
```

## 7. Les bindings `.iri` ne se résolvent qu'au **fichier d'ENTRÉE**

`import serde_json` dans un sous-module **ne suffit pas**. Importe tes **crates** dans le `main.irn` (fichier
passé à `ironc run`) ; leurs fonctions deviennent **globales** pour tous les modules fusionnés. (Les modules
Iron `import geometry`, eux, se fusionnent à plat depuis n'importe où.)

## 8. `ironc add` écrit dans le **dossier courant**

`ironc add <crate>` génère `<crate>.gen.iri` + `<crate>.iri.report.md` **dans le CWD**. Lance-le depuis le
dossier de ton `.irn` (la résolution d'`import` cherche à côté du source, dans le parent, et dans le CWD).

## 9. `str` vs octets

Iron passe une chaîne `str` là où Rust attend `&str`, `AsRef<Path/str/[u8]>`, `Into<String>`. Pour des
**octets bruts** (`list[u8]`), certains bindings attendent une liste d'entiers ; le binder coerce les éléments
(`[104, 105]` i64 → `Vec<u8>`). Si un binding veut des octets de **taille fixe**, c'est matérialisé via
`try_into()` (sentinelle `@arr`).

## 10. Couverture du binder ≠ usabilité

`ironc add` affiche une **couverture %** = fraction de l'API auto-bindée. Un crate à **41 %** (ex. nalgebra)
**marche** très bien si les items dont tu as besoin sont liés. Inversement, lis le **rapport** (`*.report.md`)
pour savoir ce qui manque avant de t'arracher les cheveux. Voir [interop.md](interop.md) et [status.md](status.md).

## 11. La couche curée `.iri` **gagne** et survit à la régénération

Si l'auto ne suffit pas, écris un `<crate>.iri` (sans `.gen`) à la main : il **écrase** l'auto par entrée, et
`ironc add` (qui ne réécrit que `<crate>.gen.iri`) **ne le touche jamais**. Voir [curation.md](curation.md).

## 12. Numérique : Python-fidèle *(V18)*

- `2 + 3.5` → **promotion** `i64→f64` automatique (résultat `f64`) sur `+ - * /`. (`**`/`//`/`%` mixtes
  restent un conflit explicite → annote `2.0`.)
- `int(x)` / `float(x)` : depuis un **numérique** → cast (`as i64` troncature / `as f64`) ; depuis une
  **chaîne** → parse. `int(3.9)` → `3`.
- `//` et `%` suivent la convention **plancher** Python (signe du diviseur) et l'identité `(a//b)*b+(a%b)==a`
  tient sur les négatifs : `-7 // 2 == -4`, `-7 % 2 == 1`.
- Index de liste **négatif** : `xs[-1]` = dernier élément (comme les slices et `str`).

## 13. Affichage : Display **Rust**, pas `repr` Python *(divergences assumées)*

Iron imprime via le `Display`/`Debug` Rust, pas le `repr` Python — quelques écarts visibles :
- `print(True)` → `true` (pas `True`) ; un **bool** s'affiche en minuscules.
- `print(1.0)` → `1` (Rust `Display` masque le `.0` d'un flottant entier ; `print(1.5)` → `1.5`).
- collections/tuples via `{:?}` → les chaînes y sont **entre guillemets** : `["a", "b"]`.
- `int("abc")` → `0` (pas d'exception ; `unwrap_or(0)`).

Ce sont des choix « projection Rust » (I6), **pas** des bugs. Pour un rendu exact, formate toi-même (f-string).

## 14. Closures : 1re classe, avec une limite de capture *(V18)*

Marchent : lambda locale, **ordre supérieur** (`apply(f, x)`), closure **retournée** (`adder(10)(5)`),
**IIFE** `(lambda x: x*2)(21)`, **champ-closure** (`obj.f(6)` où `f: fn(i64)->i64` → stocké en `Box<dyn Fn>`),
lambdas non-capturantes dans une liste. **Limite (0.1%)** : une **liste de closures capturantes** hétérogènes
(`[lambda x: x+base, …]`) exige `Vec<Box<dyn Fn>>` non émis → **contourne** par un champ de struct, ou des
lambdas non-capturantes.

## 15. `thread.scope` : lecture partagée sans clone *(V18)*

`thread.scope(lambda s: …)` ouvre un scope ; `s.spawn(…)` à l'intérieur crée des workers qui **rejoignent**
en fin de scope. Les données partagées en **lecture** (`data`, `scene`…) sont **empruntées** (`&data`) — **pas
clonées par worker**. État **mutable** partagé → `ref_sync`/`lock_add`/`lock_get` (Arc&lt;Mutex&gt;). Le corps du
scope est une **mono-expression** (typiquement une compréhension de spawns) ; une orchestration plus riche →
`rust { … }`.
