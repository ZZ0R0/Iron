# 26 — Calculatrice GUI
> **Domaine** GUI desktop · **Crate(s)** `ironc add eframe` (`egui`) · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Une calculatrice à fenêtre : grille de boutons (chiffres, opérateurs), un afficheur, évaluation à l'appui sur `=`. La **logique** (parser/évaluer une expression) est triviale en Iron ; c'est la **coquille GUI** qu'on teste vraiment.

## Ce que ça teste dans Iron
egui est en **mode immédiat**, piloté par **winit** : quatre points cognent le modèle Iron simultanément —
- **Boucle d'événements** : `eframe::run_native` rend la main à winit, qui **rappelle** ta méthode `update` à **chaque frame**. Iron n'a pas de place pour « du code que le runtime rappelle en boucle ».
- **Trait object applicatif** : il faut `impl eframe::App for App` avec `update(&mut self, ctx, frame)` — implémenter un **trait étranger** dont la signature mute un `&mut self` **persistant entre frames**.
- **Closures imbriquées** : l'UI se construit dans `CentralPanel.show(ctx, |ui| { … })`, closure **capturant `&mut self`** — closures + emprunt mutable capturé = double point faible (binder + sémantique par valeur).
- **`Box<dyn …>`** : `run_native` prend une **closure** qui rend un `Box<dyn App>` (trait object boxé).

## Approche (honnête)
Quasi **pas du Iron pur**. La logique de calcul s'écrit en Iron ; la coquille (`impl App` + `run_native` + les `|ui| …`) réclame un **`.iri` à la main** exposant les builders à closure d'egui, voire un bloc **`rust { }`** hébergeant le `impl App`. On *teste précisément jusqu'où* le binder/codegen tient avant de basculer dans l'interop manuelle.

## Risque / limites attendues
🔴 frontière : l'**état mutable que le framework possède et re-rappelle frame après frame** (l'afficheur, l'accumulateur) n'a pas d'équivalent dans un langage *par valeur* et *sans état global mutable* ; les **closures capturant `&mut self`** ne se laissent pas auto-cloner (I3 : cloner romprait la persistance de l'état entre frames). Verdict attendu : faisable **uniquement** via wrapper `.iri`/`rust { }`, pas par binding auto.

## Done
Fenêtre affichant les boutons ; `2`, `+`, `3`, `=` montre `5` ; build `cargo` vert. Si la coquille ne tient qu'en `rust { }`, **le documenter** comme limite mesurée (la frontière, c'est le livrable).
