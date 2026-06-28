# 24 — Résolveur DNS (requêtes A/MX)
> **Domaine** Réseau bas-niveau · **Crate(s)** `ironc add hickory-resolver` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Résoudre un nom de domaine en adresses **A/AAAA** et lister ses enregistrements **MX** (serveurs mail + priorité). Entrée : un domaine ; sortie : les IPs et les MX triés par préférence.

## Ce que ça teste dans Iron
- Une API à **configuration par builder** (`ResolverConfig`, `ResolverOpts`, ou `from_system_conf`) — le binder doit chaîner ces constructeurs (point faible connu sur builders/génériques).
- Des résultats en **itérateurs paresseux d'enregistrements** : `lookup_ip` et `mx_lookup` rendent des collections lazy à parcourir → pile dans la zone où le binder « galère » (API lazy). Parcours + extraction `(préférence, échange)` en `list`/compréhension.
- Le typage des records (`Name`, `RData`) que l'inférence pilotée par les sites d'appel (I15) doit relier.

## Approche
Construire un `Resolver` **synchrone** (config Google/Cloudflare ou système), `lookup_ip(domaine)`, itérer les IPs ; `mx_lookup(domaine)`, itérer en extrayant priorité + échange. Privilégier l'API **bloquante** pour éviter un runtime async.

## Risque / limites attendues
🟡 : si la version récente n'expose qu'un **`TokioAsyncResolver`**, il faut un **runtime tokio** + `block_on` (pont async = friction réelle, mais Iron a tokio). Les **itérateurs de records lazy** et les **builders de config** sont l'autre source de frottement pour le binder → un `.iri` ciblant `Resolver::new`/`lookup_ip`/`mx_lookup` est probable. Pas de boucle d'événements ni de closure → ça reste 🟡.

## Done
`resolve example.com` affiche au moins une IP A et, pour un domaine pourvu de mail, ses MX triés par priorité ; pas de panique sur domaine inexistant (`Err` géré). Build vert.
