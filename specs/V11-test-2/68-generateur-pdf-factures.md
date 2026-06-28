# 68 — Générateur de PDF (factures)
> **Domaine** Géo, temps & docs · **Crate(s)** `ironc add printpdf` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Générer une **facture en PDF** : en-tête, lignes d'articles positionnées, total, traits de séparation, éventuellement un logo. Sortie ouvrable dans n'importe quel lecteur.

## Ce que ça teste dans Iron
- Une API **à handles/index** : `PdfDocument::new` rend un document **et** des index de page/calque ; on dessine via `use_text`, on enregistre par un `BufWriter`.
- **Mutation au travers de handles** vs la **sémantique par valeur** d'Iron — le point de tension central.
- Chargement de police, newtype d'unités `Mm(f64)`, `Result` à la sauvegarde, IO `std` (`BufWriter`).

## Approche
Créer le document, une page A4, un calque, charger une police intégrée, écrire l'en-tête puis les lignes en coordonnées **mm**, tracer les filets, calculer/écrire le total, sauvegarder. Données d'entrée → PDF déterministe.

## Risque / limites attendues
- 🟡 Le **modèle à index** (`PdfLayerIndex`, références qui repointent dans le document) est peu naturel en Iron par valeur : nombreux petits index typés à faire circuler → `.iri` à soigner.
- **Builder + état mutable** porté par des handles : exactement le genre d'API que le binder rend mal.
- Sortie **binaire** : difficile d'assertion fine au-delà de « le lecteur l'ouvre / le texte est présent ».

## Done
- Un `.pdf` valide qu'un lecteur ouvre, avec en-tête, lignes et total corrects.
- Rendu **déterministe** à entrées égales.
- Zéro `unsafe`, sortie `std`.
