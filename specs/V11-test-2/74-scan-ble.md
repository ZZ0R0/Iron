# 74 — Scan d'appareils Bluetooth LE
> **Domaine** Embarqué & matériel · **Crate(s)** `ironc add btleplug` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Scanner les périphériques Bluetooth Low Energy à portée et lister leurs nom, adresse et RSSI. Le PC agit en
**central** BLE via son adaptateur intégré : pas de firmware, juste de l'écoute radio côté hôte.

## Ce que ça teste dans Iron
`btleplug` est `std` (pas de mur `no_std`/`unsafe`), mais c'est de l'**async lourd** :
- tout est **`async` + tokio** : `Manager`, `Adapter::start_scan`, et surtout un **flux d'événements**
  (`events()` → `Stream` de `CentralEvent`) ;
- des **traits async** (`Central`, `Peripheral` via `async_trait`) et des **trait-objects** — le binder V8
  galère exactement là (futures, `Stream`, méthodes async sur trait-object) ;
- des backends par plateforme (BlueZ/DBus sous Linux, CoreBluetooth, WinRT) → **adaptateur BT réel** requis.

## Approche
Init `Manager` → adaptateur → `start_scan` → consommer le `Stream` d'événements `DeviceDiscovered`, et pour
chaque périphérique lire ses propriétés (nom local, RSSI). Le nœud dur est d'**exprimer la consommation d'un
`Stream` async** depuis Iron : un `.iri` manuel enrobant le stream en itérateur async simplifié sera
probablement nécessaire.

## Risque / limites attendues
🔴 — frontière **async lourd**, pas frontière `unsafe`. Le wall est le combo **`Stream` + `async_trait` +
trait-objects** que le binder ne sait pas dériver, doublé de la dépendance à un adaptateur BT. Bon test de
la maturité de l'async d'Iron **au-delà** du simple `await` séquentiel.

## Done
Programme async qui démarre un scan, écoute le flux d'événements quelques secondes et imprime la liste des
appareils BLE découverts (nom + RSSI), sur une machine pourvue d'un adaptateur Bluetooth fonctionnel.
