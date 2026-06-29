# planify-7.1 — Curation du tail : `.iri` manuels pour ce que C1–C4 ne couvrent pas

- Réalise (cadrage) : [50400] (couche curée, niveau I7-2)
- Version : V15 · Vague : 7 · Piste : 1 · Statut : todo
- Dépend de : planify-6.1 (moteur unifié → la couverture dit ce qui reste **réellement** au tail) — **‖ planify-7.2**

## But
Pour les crates qu'**aucune** capacité auto (C1–C4) ne couvre — ce que le **rapport** [50500] liste comme
sauté après l'unification —, écrire la **couche curée** `<crate>.iri` à la main. C'est le **filet sanctionné**
du modèle Nim : on **vise** zéro binding manuel, mais quand le moteur **sait** qu'il ne peut pas (et l'a dit),
la curation manuelle est la réponse propre, **pas** un hack.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/bindings/<crate>.iri` (un fichier curé **par crate** du tail). **Aucun** fichier `ironc/src/*` —
  donc strictement disjoint de toute piste moteur et de 7.2.

## Interface produite / consommée
**Consommée** : le `<crate>.iri.report.md` de chaque crate du tail (la liste des sautés + les `hint`).
**Produite** : des entrées `.iri` curées qui, par last-wins ([50400]), complètent/écrasent l'auto.

## Étapes
1. Lire le rapport de chaque crate encore non vert après vague 6 ; trier par `SkipReason`.
2. Pour chaque item sauté **réellement nécessaire** au programme cible, écrire l'entrée `.iri` curée (signature
   Iron + chemin Rust `= "..."`), en suivant le `hint` du rapport.
3. Vérifier qu'aucune curation n'est **orpheline** (le rapport ne doit plus signaler `OrphanCuration`).
4. **Ne PAS** ajouter de nom de crate dans `ironc/src/*` — toute la spécificité reste **donnée** dans le `.iri`.

## Definition of Done (vérifiable)
- Chaque crate ciblé du tail a un `.iri` curé et un **programme témoin** qui compile + run.
- Le compteur global de projets « sans matériel » qui tournent **monte** (consigné dans CARTE-80 par 7.2).
- Gate VERT ; 0 `unsafe`.

## Notes
- **Honnêteté de périmètre** : on cure ce qui est **nécessaire** aux projets du corpus, pas l'API entière d'un
  crate. Un `.iri` curé partiel + son rapport = état **explicite**.
- Si curer un item exige du `rust { }` inline (lifetimes essentiels, macros) → c'est le niveau I7-1, à noter,
  pas à forcer en binding.
