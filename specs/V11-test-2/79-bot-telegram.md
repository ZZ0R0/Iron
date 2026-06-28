# 79 — Bot Telegram (commandes, async)
> **Domaine** Langages & exotiques · **Crate(s)** `ironc add teloxide tokio` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un bot Telegram qui répond à des commandes (`/start`, `/help`, écho…). Le bot interroge l'API Telegram en
long-polling et réagit aux messages reçus. Async de bout en bout.

## Ce que ça teste dans Iron
`teloxide` (sur `tokio`) empile **tout ce qui fait peur au binder** :
- **async lourd** : runtime `#[tokio::main]`, `bot.send_message(..).await`, trait `Requester` ;
- un **DSL de dispatch par injection de dépendances** : `dptree::entry().branch(..)`,
  `Update::filter_message`, handlers dont les arguments typés sont **injectés par type** — une magie
  difficile même en Rust pur ;
- des **macros** (`#[derive(BotCommands)]`) et des **builders** (`Dispatcher::builder`) ;
- des combinateurs génériques en cascade → le binder V8 ne peut pas dériver ça automatiquement.

## Approche
La voie teloxide est une **frontière** : il faudrait un `.iri` largement écrit à la main, et même ainsi le
`dptree` par DI résiste. **Repli honnête** : un bot « à la main » sur `ureq`/`reqwest` contre l'API HTTP de
Telegram (`getUpdates` en long-poll + `sendMessage`) — on retombe alors en 🟡, async léger + JSON, bien
dans le domaine d'Iron.

## Risque / limites attendues
🔴 — mur **async + framework macro/DI**, pas mur `unsafe`. teloxide tel quel est hors de portée du binder ;
la valeur du test est de **mesurer l'écart** entre l'async simple (gérable) et un framework async
idiomatique (ingérable automatiquement) — la même leçon que 19/25 côté serveur.

## Done
Soit un bot teloxide minimal répondant à `/start` (si le binder tient — peu probable), soit, repli assumé,
un bot équivalent sur HTTP brut qui poll les updates et répond aux commandes — en **documentant** laquelle
des deux voies a tenu.
