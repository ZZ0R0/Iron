# 43 — Visualiseur de spectre (FFT → ASCII)
> **Domaine** Audio & signal · **Crate(s)** `ironc add rustfft hound` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Lire un WAV, prendre une fenêtre d'échantillons, en calculer la FFT, puis afficher le spectre de magnitudes sous forme de barres ASCII dans le terminal.

## Ce que ça teste dans Iron
- Le **pattern planner** de rustfft : `FftPlanner::new()` puis `plan_fft_forward(n)` qui rend un `Arc<dyn Fft<f32>>` (objet-trait derrière un `Arc`) — une factory qui retourne un trait-object, à mapper proprement.
- Le type **`Complex<f32>`** (struct générique de `num-complex`, ré-exportée) : construction, accès aux champs `re`/`im`, calcul de magnitude.
- Le traitement **en place** sur un `&mut [Complex<f32>]` (`fft.process`), qui demande à Iron de passer un buffer mutable — frottement léger avec la sémantique par valeur.
- La réutilisation d'un même plan sur plusieurs fenêtres (le plan est un `Arc`, partageable) : teste le clonage/partage d'un `Arc` côté Iron.

## Approche
`ironc add rustfft hound`. Lire les échantillons via hound, les convertir en `Complex` (partie imaginaire nulle), appliquer une fenêtre (Hann), planifier puis exécuter la FFT en place, calculer la magnitude de chaque bin, normaliser et tracer une colonne de caractères par bande de fréquences. Cadrer la taille de FFT à une puissance de deux et glisser la fenêtre pour un spectre qui défile.

## Risque / limites attendues
- `Complex<f32>` et `Arc<dyn Fft>` : le binder auto peut hésiter sur le trait-object et la struct générique — un `.iri` à la main pour `Complex` + `plan_fft_forward` est probable.
- Le `process(&mut [...])` en place s'accommode mal du « tout par valeur » : il faut un buffer mutable bien identifié, pas une copie.
- Pas d'état global requis (le planner reste local) → pas de mur, juste de la friction de binding.
- Échelle des magnitudes (linéaire vs **dB**) à choisir pour un rendu lisible — décision d'affichage, pas une limite du langage.

## Done
`ironc run` lit un WAV et affiche un spectre ASCII stable ; les pics tombent sur les bonnes fréquences pour un signal de test connu (un sinus à 440 Hz pique au bon bin).
