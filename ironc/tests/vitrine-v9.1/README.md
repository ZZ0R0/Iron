# Vitrine V9.1 — l'expérience développeur Iron en un projet

Projet **multi-fichiers** qui exerce, ensemble, tout ce que V9.1 ajoute — et qui repose sur le
principe **D1 : ZÉRO bibliothèque Iron** (tout vient d'un `import`).

| Fichier | Rôle |
|---|---|
| `mathx.irn` | module Iron (`import mathx`, 70200) : un **générique** (`pick_first[T]`) + une **closure retournée** (`adder`). |
| `main.irn` | orchestre : **erreurs typées** `Result[str, CfgError]` (4.2), **crate réel** `import strsim` (distance d'édition), **concurrence** (deux threads), **`import std`** (round-trip fichier). |
| `strsim.iri` | binding auto-généré (`ironc add strsim`) — aucune lib Iron. |

## Sortie attendue (déterministe)

```
15          # générique pick_first(5,99)=5 → closure add10(5)=15
iron        # validate("iron") → ok
ERR         # validate("") → raise CfgError.Empty → unwrap_or
3           # strsim::levenshtein("kitten","sitting")
5050        # somme parallèle 1..100 sur 2 threads
vitrine-ok  # std::fs write→read round-trip
```

## Ce que ça prouve

- **Aucune lib Iron** : `mathx` (Iron), `strsim` (crate Rust), `std` — tout par `import`.
- **Profondeur** : génériques utilisateur · erreurs typées `Result[T, E]` · closures stockées.
- **Concurrence** native, **0 `unsafe`** dans le Rust généré (I2).
- Lancé par le **gate V9.1** (`tests/gate-v9.1.sh`, section [6]) : compile + run + oracle + check `unsafe`.

```sh
ironc run tests/vitrine-v9.1/main.irn      # exécute
ironc build --debug tests/vitrine-v9.1/main.irn   # + source-map I8 pour le debug
```
