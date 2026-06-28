# planify-9.1 — Émission **`impl ExternTrait for IronType`**

- Réalise (cadrage) : [24300], [44000]
- Version : V12 · Vague : 9 · Piste : 1 · Statut : todo
- Dépend de : planify-8.1 (couche d'émission), planify-6.1 (les méthodes prennent souvent des closures/&mut)

## But
Émettre un bloc `impl <trait externe> for <type Iron>` avec les **bonnes signatures** tirées des bindings du
trait (RACINE 2). Iron *parse* déjà `impl T for Ty` mais émet un Rust faux (chemins/signatures). Débloque les
frameworks à trait-utilisateur (eframe::App, iced::Application, plotters backend).

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/lower.rs` (émission de l'`impl` : chemin du trait, signatures des méthodes, `&mut self`),
  `ironc/src/bind.rs` (exposer la signature d'un trait externe implémentable — méthodes + récepteurs).
- Séquentielle (vague 9).

## Interface produite / consommée
- **Consomme** : bindings du trait externe (signatures), couche d'émission (8.1).
- **Produit** : `impl eframe::App for MyApp { fn update(&mut self, ctx: &Context, …) { … } }` correct.

## Étapes
1. binder : pour un trait externe « implémentable », mémoriser ses méthodes (nom, récepteur, params, retour).
2. lower : à `impl ExternTrait for IronType`, émettre le chemin complet du trait + chaque méthode avec la
   signature **exacte** (récepteur `&mut self`, types des params coercés via le pont).
3. Le corps utilisateur (Iron) est lowered normalement dans la méthode.

## Definition of Done (vérifiable)
1. `impl App for MyApp` (eframe) **compile** (le *run* = écran → racine 3 / environnement).
2. **Gate VERT** ; 0 `unsafe`.

## Notes
- Le *lancement* (`eframe::run_native(app)`) relève de la racine 3 (event-loop, 12.1). Ici on rend l'`impl` correct.
