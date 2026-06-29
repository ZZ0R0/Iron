# Curer un binding `.iri` à la main

Quand `ironc add` ne suffit pas (le rapport liste des items sautés dont tu as besoin), tu écris un
**`<crate>.iri`** curé. Il **complète/écrase** l'auto par entrée et **survit** aux régénérations. C'est le
niveau d'interop 2 (I7) — sanctionné, pas un hack.

## Format `.iri` (l'essentiel)

```
crate serde_json = "1.0"                 # dépendance Cargo (nom publié, version)
crate serde = "1" features [derive]      # avec features

extern fn read_file(path: str) -> Result[str] = "std::fs::read_to_string"   # fn libre → chemin Rust

extern type Point = "geo::Point":        # type externe à API (bloc indenté)
    field x: f64                         # champ public (construction par littéral)
    field y: f64
    fn new(x: f64, y: f64) -> Point = "geo::Point::new"          # ctor (pas de self)
    fn dist(self, o: Point) -> f64 = "distance"  [clone]         # méthode (self) + idiome

extern type Vector3[T] = "nalgebra::Vector3":   # type GÉNÉRIQUE appliqué
    base Matrix                          # hérite les méthodes de `Matrix` (un Vector3 EST un Matrix)
    fn new(x: T, y: T, z: T) -> Vector3[T] = "nalgebra::Vector3::new"

extern const PI: f64 = "std::f64::consts::PI"   # constante publique
extern trait App = "eframe::App"                # trait implémentable (class My(App): …)
```

- **Récepteur** : 1er param `self` (`&self`), `mut self` (`&mut self`), `own self` (`self` par valeur),
  ou rien (= constructeur).
- **`own T`** sur un param = passé par valeur (pas de `&`).
- **Idiomes** entre crochets : `[builder]` `[clone]` `[collect]` `[closure]` `[display]`.
- **`base Nom`** dans un bloc type = héritage des méthodes du type de base (alias → type nominal).
- **first-wins / last-wins** : une entrée curée `<crate>.iri` écrase l'auto `<crate>.gen.iri`.

## Matérialisation par **chemin-closure**

Quand l'appel direct ne suffit pas (conversion, type intermédiaire), le « chemin Rust » peut être une
**closure** qui enveloppe les arguments :

```
fn new(key: list[u8]) -> Aes256Gcm = "(|k: &[u8]| <aes_gcm::Aes256Gcm as aes_gcm::KeyInit>::new(k.try_into().unwrap()))"
```
Iron passe les arguments à la closure ; tu fais la conversion (`try_into`, `from_slice`, `into`…) à l'intérieur.

## Gabarit complet : aes-gcm (cas AEAD réel)

L'auto-AEAD n'est pas généralisable (le nonce `Array<u8,N>` et `impl Into<Payload>` sont durs, et reconnaître
`Array<u8,N>` casserait ndarray). Curation propre (`aes_gcm.iri`) — **vérifiée par run** (round-trip) :

```
crate aes-gcm = "0.11.0"
crate aead = "0.6.1"

extern type Aes256Gcm = "aes_gcm::Aes256Gcm":
    fn new(key: list[u8]) -> Aes256Gcm = "(|k: &[u8]| <aes_gcm::Aes256Gcm as aes_gcm::KeyInit>::new(k.try_into().unwrap()))"
    fn encrypt(self, nonce: list[u8], plaintext: list[u8]) -> Result[list[u8]] = "(|c: &aes_gcm::Aes256Gcm, n: &[u8], p: &[u8]| aes_gcm::aead::Aead::encrypt(c, n.try_into().unwrap(), p))"
    fn decrypt(self, nonce: list[u8], ciphertext: list[u8]) -> Result[list[u8]] = "(|c: &aes_gcm::Aes256Gcm, n: &[u8], ct: &[u8]| aes_gcm::aead::Aead::decrypt(c, n.try_into().unwrap(), ct))"
```
Programme :
```python
import aes_gcm
def run() -> Result[i64]:
    cipher = Aes256Gcm.new([0 for _ in range(32)])
    ct = cipher.encrypt([0 for _ in range(12)], [104, 105, 33])?
    pt = cipher.decrypt([0 for _ in range(12)], ct)?
    return len(pt)
def main():
    try:
        print(run()?)          # 3
    except:
        print("err")
```

(Exemples prêts à l'emploi : `specs/V15/examples/` — `aes_gcm.iri`, `num_rational.irn`, `arrayvec_const_cap.irn`…)

## Workflow de curation

1. `ironc add <crate>` → lis `<crate>.iri.report.md` (ce qui manque + pourquoi).
2. Crée `<crate>.iri` ; binde **uniquement les items dont ton programme a besoin** (pas toute l'API).
3. Pour les conversions, utilise un **chemin-closure** (cf. ci-dessus).
4. `ironc run` ton programme ; les erreurs `rustc` remontent sur ta ligne Iron pour t'orienter.
5. Si un item exige des lifetimes essentiels ou des macros → `rust { }` inline plutôt que `.iri`.

> **Ne cure pas une API entière** : seulement ce qui sert. Un `.iri` curé partiel + son rapport = état explicite.
