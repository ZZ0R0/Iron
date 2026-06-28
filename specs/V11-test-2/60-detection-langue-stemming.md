# 60 — Détection de langue + racinisation (stemming)
> **Domaine** ML & IA · **Crate(s)** `ironc add whatlang` · `ironc add rust-stemmers` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Détecter la langue d'un texte (`whatlang`) puis réduire ses mots à leur racine avec l'algorithme de
Porter/Snowball adapté à cette langue (`rust-stemmers`). Imprimer langue + confiance + mots racinisés.

## Ce que ça teste dans Iron
Cas **feuille** par excellence pour le binder, côté ML « léger » (pas de tenseurs) : API faite de
**fonctions pures** et d'**enums**. `whatlang::detect(text) -> Option<Info>` (avec `lang()`, `script()`,
`confidence()`), et `Stemmer::create(Algorithm::English).stem(mot)`. On teste donc le **mapping d'enums**
(`Lang`, `Script`, `Algorithm`), le retour **`Option`**, et la coercition d'un retour **`Cow<str>`** vers
une `String` Iron. Aucun générique, aucun état global.

## Approche
`ironc add whatlang` et `ironc add rust-stemmers`, `import`, puis : `detect()`, lire `info.lang()`,
construire **une fois** le `Stemmer` correspondant et `stem()` chaque mot dans une boucle. Choisir
l'`Algorithm` à partir de la `Lang` détectée (petit `match`). Cas e2e : un paragraphe FR et un EN, vérifier
langue correcte + quelques racines connues (`running → run`).

## Risque / limites attendues
- `stem()` rend un **`Cow<str>`** : le binder doit le coercer en `String` possédée (friction mineure).
- Variantes d'**enums** (`Algorithm::French`, `Lang::Eng`) à mapper — trivial mais à couvrir.
- `detect` rend une **`Option`** (texte trop court) → un `if`/`match` côté Iron. Risque global faible.

## Done
`ironc run` identifie FR et EN avec une confiance correcte et imprime des mots racinisés justes. Compile
sans `unsafe` ; c'est un témoin 🟢 du binder sur enums + fonctions pures.
