# Iron — extension VSCode

Support du langage **Iron** (`.irn`/`.iri`) : coloration, diagnostics live (LSP + overlay rustc), imports de
crates Rust, build/run.

## Ce qu'elle apporte
- **Fichiers reconnus** `.irn`/`.iri` + **icône** dans l'explorateur (`Fe`).
- **Coloration** syntaxique (mots-clés, types, chaînes, f-strings, décorateurs `@`).
- **Diagnostics live** : LSP (`ironc lsp`, lex/parse instantané) + `ironc check` au save (overlay rustc).
- **Commandes** (palette) : `Iron: Add crate` (importe un crate Rust), `Build`, `Run`, `Test`, `Format`.
- **CodeLens** « ▶ Run » au-dessus de `def main()`.

## Pré-requis
Le binaire **`ironc`** dans le `PATH` (ou configurer `iron.path`).

## Installation (dev)
```bash
cd editors/vscode
npm install
npm run compile
npm run package     # → iron-lang-0.1.0.vsix
code --install-extension iron-lang-0.1.0.vsix
```

## Configuration
- `iron.path` : chemin du binaire `ironc` (défaut `ironc`).
- `iron.checkOnSave` : lancer `ironc check` (overlay rustc) à la sauvegarde (défaut `true`).
