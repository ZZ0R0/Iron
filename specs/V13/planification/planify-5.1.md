# planify-5.1 — Retour zéro-copie tenu (`Smf<'a>`, events) → midly, quick-xml ⚠️ PROTOTYPE

- Réalise (cadrage) : [30200] niveau 2
- Version : V13 · Vague : 5 · Piste : 1 · Statut : todo (**prototype — tranche difficile vs incompatible**)
- Dépend de : planify-2.1 (niveau 1 fait)

## But
Gérer un retour qui **tient un emprunt zéro-copie de l'entrée** (`midly::Smf<'a>` parse sans copier ; un event
quick-xml référence le buffer du reader). Iron par valeur ne le permet pas directement.

## ⚠️ Gate prototype (AVANT planif atomique détaillée)
Prototype d'abord : pour midly/quick-xml, existe-t-il une voie **clone-vers-possédé** (`Smf::make_static`/
`event.into_owned()`) qui rend une valeur **sans lien de durée de vie** ? Si oui → idiome `owned-clone`. Si non
→ il faudrait **garder l'entrée vivante** (suivi de durée de vie partiel) — ce qui peut être **incompatible**
avec la sémantique par valeur. **Le prototype tranche ; on ne planifie le détail qu'après.**

## Fichiers POSSÉDÉS (sous réserve du prototype)
- `ironc/src/bind.rs` : détecter un retour zéro-copie + une méthode `*_owned`/`make_static` associée → idiome
  qui chaîne la conversion possédée.
- `ironc/src/lower.rs` : émettre `(call).into_owned()`/`.make_static()`.

## Interface produite / consommée
- **Consomme** : idiomes (V13 2.1).
- **Produit** : parse midly/quick-xml rendant une valeur possédée (si la voie clone-owned existe).

## Étapes (post-prototype)
1. Prototype : `midly::Smf::parse(bytes)` → `.make_static()` rend un `Smf<'static>` ? quick-xml event `.into_owned()` ?
2. Si oui : idiome `owned` (retour zéro-copie + conversion possédée connue → chaîner).
3. Tester : midly parse un MIDI (octets) → compte les tracks ; quick-xml lit des events.

## Definition of Done (vérifiable)
1. midly : parse un fichier MIDI (octets) → nb de pistes (run réel) **si** `make_static` suffit.
2. **Sinon** : limite documentée — zéro-copie tenu **incompatible** avec la sémantique par valeur (comme
   event-loop : tranché par l'essai, pas caché).
3. Gate VERT 24/0 ; 0 `unsafe`.

## Notes
- Honnête : c'est le cas le **plus incertain** de V13. On l'essaie, on tranche, on documente. Pas de promesse.
