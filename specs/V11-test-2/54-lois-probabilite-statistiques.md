# 54 — Lois de probabilité & statistiques
> **Domaine** Maths/vectoriel · **Crate(s)** `ironc add statrs` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Instancier des lois usuelles (Normale, Binomiale, Poisson), évaluer densité/répartition (`pdf`, `cdf`,
`inverse_cdf`) et lire moyenne/variance/écart-type. En appui : quantiles, statistiques d'un échantillon.

## Ce que ça teste dans Iron
API **concrète** (un struct par loi) → bon pour le binder, **mais** une subtilité : les méthodes
`pdf`/`cdf`/`mean` viennent de **traits** (`Continuous`, `ContinuousCDF`, `Distribution`), pas du struct
lui-même. Le binder doit donc **surfacer les méthodes de trait** pour les rendre appelables. Les
constructeurs rendent un `Result` (paramètres invalides) → on teste la **gestion d'erreur** côté Iron.

## Approche
`ironc add statrs`, `import`, créer `Normal::new(mu, sigma)` (gérer le `Result`), appeler `dist.pdf(x)` /
`dist.cdf(x)` / `dist.mean()`. Rester sur les évaluations **déterministes** (pdf/cdf/quantiles, stats
d'échantillon) pour garder le 🟢. Cas e2e : vérifier `cdf(μ) ≈ 0.5` pour une Normale et quelques valeurs
tabulées d'une Binomiale.

## Risque / limites attendues
- **Méthodes de trait** : si le binder ne lie que les méthodes inhérentes, `pdf`/`cdf` manqueront → il
  faut déclarer les traits explicitement dans le `.iri` (friction modérée, pas un mur).
- L'**échantillonnage** (`sample`) exige `rand` : un `&mut Rng` (état mutable) qui cogne avec la sémantique
  par valeur → **hors périmètre** pour rester vert ; on s'en tient au déterministe.
- Constructeurs en `Result` : à mapper proprement sur la gestion d'erreur Iron.

## Done
`ironc run` imprime `pdf`/`cdf`/moyenne/variance d'une Normale et d'une Binomiale, valeurs vérifiées
contre des tables. Compile sans `unsafe`.
