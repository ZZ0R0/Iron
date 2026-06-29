# 71 — Calculateur de durées et dates (crate `time`)
> **Domaine** Temps · **Crate(s)** `ironc add time` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Construire des dates, ajouter une durée (`Duration`), calculer l'écart entre deux dates, formater.

## Ce que ça teste dans Iron
`time` (alternative à chrono) a une API à **builders et types associés** (`Date`, `Time`, `OffsetDateTime`,
`Duration`) et des **macros** de construction. Teste un second crate temps (API différente de chrono déjà ✅)
et le mappage de `Duration` + opérateurs.

## Approche
`Date.from_calendar_date(2026, Month, day)` (ou ctor concret), `+ Duration.days(n)`, écart en jours.
Cibler les ctors non-macro.

## Done
`ironc run` imprime l'écart en jours entre deux dates (valeur connue).
