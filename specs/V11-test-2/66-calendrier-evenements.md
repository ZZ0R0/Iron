# 66 — Calendrier / planificateur d'événements
> **Domaine** Géo, temps & docs · **Crate(s)** `ironc add chrono` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Gérer des événements datés (titre, début, fin), les lister par ordre chronologique, filtrer « les 7 prochains jours » et **détecter les chevauchements** d'agenda.

## Ce que ça teste dans Iron
- L'interop avec une API **value-semantic** idéale pour Iron : `chrono` manipule `NaiveDateTime`/`DateTime` et `Duration` par valeur.
- **Parsing/formatage** via motifs `strftime` (`parse_from_str` / `format`).
- **Surcharge d'opérateurs** (date `+` `Duration`), **comparaison/tri** d'un `Vec` d'événements.

## Approche
Une classe `Event` (titre, début, fin). Lire des dates depuis des chaînes, stocker dans un `Vec`, trier par début, requêter la fenêtre « 7 jours », détecter les chevauchements (deux intervalles qui s'intersectent). Persistance optionnelle en JSON (`serde`).

## Risque / limites attendues
- 🟢 `chrono` est ergonomique et par valeur : très bon fit Iron.
- Petit point : le **trait `TimeZone`** rend `DateTime<Utc>` vs `DateTime<Local>` génériques — rester sur `NaiveDateTime` ou `Utc` évite la friction.
- Les motifs `strftime` sont des chaînes runtime : une erreur de format est une `Err`, pas une erreur de compile (limite assumée).

## Done
- Ajout d'événements, tri chronologique correct, filtre « à venir » exact.
- Détection de chevauchement correcte sur des cas limites (bords jointifs).
- Aller-retour fichier (si JSON) sans perte.
