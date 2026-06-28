# planify-11.1 — Runtime **async** (binder + await des fns async) — ⚠️ GATE PROTOTYPE

- Réalise (cadrage) : [53000]
- Version : V12 · Vague : 11 · Piste : 1 · Statut : **todo (gate prototype d'abord)**
- Dépend de : planify-3.1 (génériques), planify-6.1 (closures/handlers)

## ⚠️ Gate prototype (AVANT toute planif atomique détaillée)
Iron a déjà `async def`/`await` en surface + injecte tokio sur `main` (`is_async`). **Prototype à faire en
premier** : binder une vraie fn async (`reqwest::get(url).await`) et la faire **tourner**. Le prototype **tranche** :
quelle part de l'écosystème async passe (appels simples) vs reste **partielle** (traits lourds `tower::Service`,
`axum::Handler` à génériques profonds). On ne planifie le détail qu'**après** ce verdict.

## But (sous réserve du prototype)
Binder les fns/méthodes **async** des crates et les `.await` correctement ; wiring runtime déjà présent sur `main`.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bind.rs` (marquer une fn externe `async`), `ironc/src/infer/` (type `Future`/résultat
  d'`await`), `ironc/src/lower.rs` (`(expr).await`, `async` propagé). `driver.rs` si features tokio à ajuster.
- Séquentielle (vague 11).

## Interface produite / consommée
- **Consomme** : pont (1.1), closures (6.1) pour les handlers.
- **Produit** : appels async de crates exécutés sous le runtime injecté.

## Étapes (post-prototype)
1. binder : signature async → fn Iron `async` ; retour `Future<T>` → `await` donne `T`.
2. lower : émettre `.await` ; s'assurer que la fonction appelante est `async` (propagation).
3. Cibler d'abord les appels **simples** (client HTTP) ; **noter** ce qui exige des traits async lourds.

## Definition of Done (vérifiable)
1. Un client HTTP async minimal (`reqwest`/`ureq`-async) **récupère** une réponse (run réel, réseau dispo).
2. **Gate VERT** ; 0 `unsafe`.

## Notes
- **Honnêteté** : axum/teloxide/hickory (serveurs, traits lourds) peuvent rester **partiels** — le prototype le dira ; ne pas survendre.
- Le réseau au run = environnement (hors langage) ; on valide la **compilation + un appel** quand le réseau est dispo.
