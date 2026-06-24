# [3.4] Source maps granularité expression (60200)

- Spec parente : 60200 · V6 · **Vague 3** · Statut : todo · Dépend de : 1.1

## But
Atteindre la cible I8 : mapping `Span` Iron ↔ Rust au niveau **expression** (V0–V5 = instruction).

## Fichiers possédés
`src/lower.rs` (marqueurs par expression), `src/sourcemap.rs` (ré-localisation fine).

## Étapes
1. Émettre un marqueur par `Expr` (pas seulement par instruction), survivant à `prettyplease`.
2. Maîtriser la **pollution** (marqueurs invisibles / retirés du binaire).
3. Remonter une erreur `rustc` au **sous-bout** exact.

## Critère de done
Erreur `rustc` sur une sous-expression → message pointant le sous-bout Iron ; robustesse reformatage ; corpus 22/22 inchangé.

## Notes
Trop de marqueurs ⇒ `prettyplease` peut les déplacer — mécanisme type `#[line]` à étudier (spec 60200).
