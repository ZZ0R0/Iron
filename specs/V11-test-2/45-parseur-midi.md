# 45 — Lecteur/parseur de fichier MIDI
> **Domaine** Audio & signal · **Crate(s)** `ironc add midly` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Parser un fichier MIDI standard (.mid), lister ses pistes et en extraire les événements note-on (hauteur, instant, canal), ainsi que le tempo et la division temporelle.

## Ce que ça teste dans Iron
- Le parsing d'un **format binaire** vers un **arbre d'enums riche** : `Smf` → `tracks: Vec<Vec<TrackEvent>>`, chaque `TrackEvent { delta, kind }`, `kind` étant un enum imbriqué (`TrackEventKind::Midi { channel, message }` → `MidiMessage::NoteOn { key, vel }`).
- Le **match d'enums profondément imbriqués** et l'itération de `Vec` de `Vec`.
- Le gros frottement : midly est **zéro-copie** — `Smf<'a>` **emprunte** le tampon d'octets d'entrée (vue liée par lifetime). La sémantique par valeur d'Iron contre une vue empruntée.
- Les newtypes `u7`/`u4` (entiers bornés) pour hauteurs/canaux.
- Le **delta-time** est décodé par midly, mais convertir ticks → secondes (via tempo + division) est de l'arithmétique à écrire — bon test de calcul entier/flottant.

## Approche
`ironc add midly`. Lire le fichier en `Vec<u8>`, `Smf::parse`, lire l'en-tête (format, division). Pour s'épargner le lifetime, basculer sur la variante **possédée** (`make_static`/`to_static`) avant de stocker. Itérer les pistes, matcher les `kind`, accumuler les delta-times pour dater chaque note-on. Filtrer sur `NoteOn` à vélocité non nulle (un `NoteOn` vel=0 vaut note-off) pour ne compter que les vraies frappes.

## Risque / limites attendues
- Le **lifetime** de `Smf<'a>` est le vrai piège : ou bien garder le `Vec<u8>` vivant aussi longtemps que le `Smf`, ou bien posséder via `to_static`. Un binder par valeur préférera l'option possédée.
- L'arbre d'enums (TrackEventKind/MidiMessage/MetaMessage) est large à mapper ; les newtypes `u7`/`u4` demandent un déballage explicite.
- Pas de boucle d'événements ni d'async → pas de mur, juste du binding minutieux.
- L'écriture (re-sérialisation `Smf::save`) ajouterait l'aller-retour complet ; rester en lecture seule borne le périmètre.

## Done
`ironc run` parse un .mid, affiche le format et la division, liste les pistes, et imprime les note-on (pitch, tick, canal) plus le tempo lu dans les méta-événements.
