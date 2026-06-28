# 70 — Convertisseur de fuseaux horaires
> **Domaine** Géo, temps & docs · **Crate(s)** `ironc add chrono-tz` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Convertir une date-heure d'un **fuseau IANA** vers un autre (ex. `America/New_York` → `Europe/Paris`), en gérant correctement l'**heure d'été** (DST). Nom de fuseau invalide rejeté.

## Ce que ça teste dans Iron
- L'enum **massive `Tz`** générée par `chrono-tz` : Iron doit la **surfacer** (sans doute via `parse` depuis une chaîne plutôt que des centaines de variantes).
- L'intégration avec le **trait `TimeZone`** de `chrono` : `DateTime<Tz>`, `.with_timezone(&tz)`.
- Parsing `"Europe/Paris".parse::<Tz>()` → `Result`, formatage de sortie.

## Approche
Lire date-heure + fuseau source + fuseau cible ; construire un `DateTime<Tz>` ; convertir vers la cible ; formater. Montrer un passage de **frontière DST**. Nom de fuseau inconnu → `Err`.

## Risque / limites attendues
- 🟢 API propre, mais l'**énorme enum `Tz`** est le vrai test du binder : préférer l'exposer via `parse`-from-str qu'énumérer ~600 variantes.
- `DateTime<Tz>` **générique** sur le trait `TimeZone` — Iron doit le porter.
- Subtilité de correction : les heures locales **inexistantes/ambiguës** (saut DST) renvoient un `LocalResult` — à exposer **honnêtement**, pas à écraser silencieusement en une valeur fausse.

## Done
- Conversion correcte entre zones, **y compris** à travers une transition DST.
- Nom de fuseau invalide → `Err` typée.
- Cas local ambigu/inexistant **signalé** (pas silencieusement résolu).
