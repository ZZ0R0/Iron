# 17 — Scraper web (extraire des éléments d'une page)
> **Domaine** Web & réseau · **Crate(s)** `ironc add scraper ureq` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Télécharger une page HTML et en extraire des éléments par sélecteur CSS : titres, liens, prix… En entrée : une URL + un sélecteur (ex. `h2.title a`) ; en sortie : la liste des textes et attributs `href` des éléments correspondants.

## Ce que ça teste dans Iron
Une API **fortement à durées de vie** : `ElementRef<'a>` emprunte le document `Html`, `Selector` se compile à part, et l'extraction passe par des **itérateurs paresseux** (`.select()`, `.text()`). Teste comment la **sémantique par valeur d'Iron** compose avec une API où tout **emprunte** un arbre partagé (`ego-tree`).

## Approche
- Fetch : `ureq::get(url).call()?.into_string()?`.
- Parse : `scraper::Html::parse_document(&body)` → `Html`.
- Sélecteur : `Selector::parse("h2.title a")?` → `Selector`.
- Extraction : `doc.select(&sel)` → itérateur d'`ElementRef` ; par élément `el.text().collect::<String>()` et `el.value().attr("href")`.

## Risque / limites attendues
- Les **lifetimes** (`ElementRef<'a>` lié à `Html`, `Selector` réutilisable) sont le cœur du risque : le binder doit garder `Html` vivant pendant l'itération → probable wrapper Iron qui **possède** le document et rend des valeurs (`String`) plutôt que des emprunts.
- `.text()` rend un **itérateur** (souvent multi-fragments) à recollecter ; `attr()` rend une `Option`.
- HTML malformé : `scraper` est tolérant, mais un sélecteur invalide doit échouer proprement (Result).

## Done
Sur une page de test, le sélecteur rend la bonne liste (texte + href) ; sélecteur sans match → liste vide, pas de panic. Cas de test : page locale avec 3 `<a class="item">`, extraire 3 couples (texte, href) attendus ; sélecteur bidon → 0 résultat.
