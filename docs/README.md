# Iron — Documentation

> **Iron** est un langage à **syntaxe Python**, **sémantique par valeur**, qui **transpile vers Rust** et
> compile via `cargo`. Pas de GC, pas de runtime lourd, pas d'`unsafe`. L'écosystème d'Iron **EST** crates.io :
> on importe directement des crates Rust (`import serde_json`), le binder génère l'interop.

Cette doc est **dense et structurée** pour deux publics :
- **humains** qui découvrent ou utilisent Iron ;
- **agents (Claude Code) en session future** qui doivent connaître vite les **subtilités** sans relire tout le source.

## Par où commencer

| # | Fichier | Pour quoi |
|---|---|---|
| 1 | [getting-started.md](getting-started.md) | Construire `ironc`, écrire et lancer un premier programme. |
| 2 | [language.md](language.md) | La syntaxe et les types par l'exemple (le tour du langage). |
| 3 | [semantics.md](semantics.md) | Sémantique par valeur, ownership/clone, mutation, **les invariants I1–I15**. |
| 4 | [interop.md](interop.md) | **Le cœur** : importer des crates Rust (`ironc add`, `.iri`, le rapport, les deux couches, les idiomes, la frontière). |
| 5 | [curation.md](curation.md) | Curer un binding `.iri` à la main quand l'auto ne suffit pas (gabarit + patterns). |
| 6 | [gotchas.md](gotchas.md) | **⚠️ Les subtilités qui piègent** — à lire avant d'écrire du Iron sérieux. |
| 7 | [cli.md](cli.md) | Référence des commandes `ironc` + flags + workflow. |
| 8 | [status.md](status.md) | État **mesuré** : ce qui marche, la frontière du binder. |

## Quickstart (60 secondes)

```bash
# 1. Construire le compilateur (une fois)
cargo build --release --manifest-path ironc/Cargo.toml
# binaire : ironc/target/release/ironc

# 2. Écrire un programme  (hello.irn)
cat > hello.irn <<'EOF'
def main():
    xs = [3, 1, 2]
    total = 0
    for x in xs:
        total = total + x
    print(total)        # 6
EOF

# 3. Le lancer (transpile → cargo run)
ironc/target/release/ironc run hello.irn
```

Pour utiliser un crate Rust :

```bash
ironc add semver           # binde le crate → semver.gen.iri + un rapport de couverture
cat > v.irn <<'EOF'
import semver
def main():
    v = Version.new(1, 2, 3)
    print(v.major)          # 1
EOF
ironc run v.irn
```

## Modèle mental en une image

```
prog.irn  ──ironc──►  lexer → parser → inférence → ownership → lowering(TokenStream)
                                                                     │
                                            crates: import X ◄── X.gen.iri (binder)
                                                                     ▼
                                                   Rust propre ──► cargo build/run
```

- **Tu écris du Python-ish**, Iron produit du **Rust idiomatique** sans `unsafe`, cargo le compile.
- **L'inférence est quasi totale** : presque aucune annotation de type à écrire.
- **Le filet, c'est `rustc`** : si l'inférence/binding se trompe, c'est une erreur Rust **remontée sur ta ligne Iron** — jamais une corruption.

> **Honnêteté** : Iron couvre très bien un large sous-ensemble (CLI, données, maths, crypto, image…), mais a
> une **frontière mesurée** (type-level lourd, event-loops, async, no_std, ML lourd). Voir [status.md](status.md).
