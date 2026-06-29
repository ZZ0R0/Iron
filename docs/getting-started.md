# Getting started

## 1. Construire le compilateur

Iron est compilé par `ironc` (écrit en Rust). Il faut **Rust stable** ; pour `ironc add` (binding de crates),
il faut aussi **Rust nightly** (rustdoc JSON).

```bash
cargo build --release --manifest-path ironc/Cargo.toml
# binaire : ironc/target/release/ironc
# pratique : export IRONC=$PWD/ironc/target/release/ironc
```

Vérifier nightly (pour `ironc add` seulement) :

```bash
rustup toolchain list | grep nightly   # sinon : rustup toolchain install nightly
```

## 2. Premier programme

`hello.irn` :

```python
def main():
    name = "Iron"
    print("salut " + name)
    total = 0
    for i in range(1, 5):       # 1,2,3,4
        total = total + i
    print(total)                # 10
```

Lancer (transpile → `cargo run` sur le Rust généré) :

```bash
$IRONC run hello.irn
```

## 3. Voir le Rust généré

```bash
$IRONC emit hello.irn          # écrit un projet cargo lisible dans hello-rust/
cat hello-rust/src/main.rs     # le Rust idiomatique produit (sans unsafe)
```

`emit` est l'esprit *nimcache* : le `.rs` intermédiaire est un artefact de premier ordre, lisible et éditable.

## 4. Utiliser un crate Rust

```bash
$IRONC add chrono              # compile chrono, génère chrono.gen.iri + chrono.iri.report.md
```

`add` affiche un **résumé de couverture** (« N liés, M sautés, P % ») et écrit un **rapport** listant ce qu'il
n'a pas su binder et pourquoi. Puis :

```python
import chrono
def main():
    d = parse_from_str("2026-06-28", "%Y-%m-%d")   # selon le binding
    print(d.weekday())
```

```bash
$IRONC run prog.irn            # trouve chrono.gen.iri dans le dossier du .irn
```

Détails du binding : [interop.md](interop.md).

## 5. Un projet structuré (optionnel)

```bash
$IRONC new mon_projet          # crée mon_projet/{src/main.irn, ironc.toml, .gitignore}
cd mon_projet
$IRONC run                     # utilise [iron] entry de ironc.toml (pas besoin de nommer le fichier)
```

`ironc.toml` (le « Cargo.toml » d'Iron) déclare l'entrée, les dépendances `ironc add`, et des options
(`[iron] emit = "clean"|"instrumented"`, `rust-out = "dir"`).

## 6. Multi-fichiers

```python
import geometry        # charge geometry.irn (module Iron), fusionné à plat
import serde_json      # charge serde_json.gen.iri (binding de crate)
```

`import X` cherche, dans l'ordre : un binding curé embarqué (`import std`), un module `X.irn`, puis un binding
`X.gen.iri`/`X.iri`. **Les bindings ne se résolvent qu'au fichier d'ENTRÉE** : importe tes crates dans le
`main.irn`, leurs fonctions deviennent globales pour tous les modules.

## Étapes suivantes
- Le langage en détail → [language.md](language.md)
- **Les pièges à connaître** → [gotchas.md](gotchas.md)
- Toutes les commandes → [cli.md](cli.md)
