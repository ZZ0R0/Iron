# planify-12.1 — **Event-loop** = callback + impl-trait + état partagé — ⚠️ GATE PROTOTYPE

- Réalise (cadrage) : [35000]
- Version : V12 · Vague : 12 · Piste : 1 · Statut : **todo (gate prototype d'abord)**
- Dépend de : planify-6.1 (closures), planify-9.1 (impl-trait externe)

## ⚠️ Correction d'un cadrage erroné + gate prototype
J'avais dit « [35000] incompatible avec la sémantique par valeur ». **Faux** : Iron a `shared[T]`=`Rc<RefCell>`,
`shared_sync[T]`=`Arc<Mutex>`, `inout`. Une boucle d'événements **se décompose** en : un **callback** (racine 1,
6.1) ou un **`impl App`** (racine 2, 9.1), capturant un **état mutable partagé** (existant). Donc pas un mur —
une **intégration**. **Prototype d'abord** : `eframe::run(|ctx| { ui(ctx, shared_state) })` doit **compiler** et
(dans un env avec écran) **afficher**. Le prototype tranche l'**ergonomie** de l'état partagé (acceptable vs trop lourde).

## But (sous réserve du prototype)
Permettre qu'un **framework possède la boucle** et rappelle du code Iron à chaque frame, avec état muté via `shared`.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/lower.rs` (intégration : passer le `shared[State]` dans le callback/`impl`, glue de
  lancement `framework::run(...)`). S'appuie sur 6.1 (closure) et 9.1 (impl) déjà faits.
- Séquentielle (vague 12, dernière).

## Interface produite / consommée
- **Consomme** : closures (6.1), impl-trait (9.1), `shared`/`shared_sync` (existant).
- **Produit** : programmes GUI/jeu/TUI dont la boucle est tenue par le framework.

## Étapes (post-prototype)
1. Lancer `framework::run(callback)` / `run_native(Box::new(app))` avec l'état partagé capturé.
2. S'assurer que la mutation par frame passe par `shared`/`shared_sync` (pas d'état global mutable — invariant respecté).
3. Cibler immediate-mode (egui) d'abord (callback) ; retain/trait (eframe/iced) via 9.1.

## Definition of Done (vérifiable)
1. Un GUI minimal (egui immediate-mode, compteur dans `shared[State]`) **compile** ; **run** validé en env avec écran.
2. **Gate VERT** ; **0 état global mutable** émis (sémantique respectée) ; 0 `unsafe`.

## Notes
- **Honnêteté** : l'ergonomie de l'état partagé sur de gros jeux peut rester lourde — le prototype le dit. Le
  *run* exige un écran (environnement, hors langage) ; on valide compile + run quand l'écran est dispo.
