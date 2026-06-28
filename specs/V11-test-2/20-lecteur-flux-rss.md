# 20 — Lecteur de flux RSS/Atom
> **Domaine** Web & réseau · **Crate(s)** `ironc add rss ureq` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Télécharger un flux RSS et en lister les derniers articles. En entrée : l'URL d'un flux ; en sortie : pour les N premiers items, titre + lien + date de publication. Gère un flux malformé sans planter.

## Ce que ça teste dans Iron
Un parseur de format à **API d'accesseurs plats** : `Channel::read_from` puis une grappe de getters rendant surtout des `Option<&str>`. Cas « feuille » idéal — le seul point technique est que `read_from` consomme un **`BufRead`**, pas une `String`.

## Approche
- Fetch : `ureq::get(url).call()?.into_string()?` → corps XML.
- Parse : `rss::Channel::read_from(body.as_bytes())?` (`&[u8]` est `BufRead`) → `Channel`.
- Lecture : `channel.title()`, `channel.items()` → `&[Item]` ; par item `item.title()`, `item.link()`, `item.pub_date()` (des `Option<&str>`).

## Risque / limites attendues
- Faible. Attention au paramètre **`BufRead`** de `read_from` : passer `body.as_bytes()` ou un `Cursor` ; le binder doit exposer la bonne surcharge.
- Les getters rendent des **`Option`** (champs facultatifs en RSS) → déballage propre côté Iron.
- **Atom** n'est *pas* couvert par `rss` (crate distinct `atom_syndication`) : si le flux est Atom, le détecter et le signaler, ou ajouter le crate — à noter comme limite de périmètre.

## Done
Parse un vrai flux RSS et liste correctement titre+lien des N derniers items ; un XML tronqué rend une erreur propre (pas de panic). Cas de test : un échantillon RSS à 3 items → 3 couples (titre, lien) attendus ; entrée non-XML → erreur gérée.
