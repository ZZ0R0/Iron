# 72 — GPIO Raspberry Pi (capteur / LED)
> **Domaine** Embarqué & matériel · **Crate(s)** `ironc add rppal` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Sur un Raspberry Pi (Linux), lire un capteur (bouton, fin de course) et/ou allumer une LED via les broches
GPIO. Le « blink » du Pi — mais **côté Linux**, pas bare-metal : un programme classique qui parle au noyau.

## Ce que ça teste dans Iron
Le contraste-clé avec l'ESP32 (71) : `rppal` est un crate **`std`** qui pilote la GPIO via le noyau Linux
(`/dev/gpiomem`, `/dev/gpiochip0`). Donc **ni `no_std`, ni `unsafe` côté utilisateur** (rppal encapsule le
bas-niveau) — les deux murs durs d'Iron tombent. Restent trois frictions réelles :
- la **cross-compilation** vers `aarch64`/`armv7` Linux (ou compilation directement sur le Pi) ;
- l'**API type-state** : `Gpio::new()?.get(pin)?.into_output()` puis `.set_high()`/`.set_low()`, où
  `into_input`/`into_output` **consomment** la valeur — la sémantique par valeur d'Iron doit suivre ces moves ;
- l'impossibilité de **smoke-tester sans matériel** (pas de Pi en CI).

## Approche
Rester sur l'API **directe** de rppal (`Gpio`/`OutputPin`/`InputPin`), pas sur les traits `embedded-hal`
génériques (cf. 75). Boucle simple : init Gpio, configurer la broche, lire/écrire, `sleep`. Si le binder
bute sur les transitions de type-state ou les `Result`, un `.iri` écrit à la main suffit à débloquer.

## Risque / limites attendues
🔴 — mais une **frontière franchissable** : le bloqueur n'est plus l'invariant `unsafe`/`no_std` (rppal est
`std`), c'est la **dépendance matérielle + cross-compile + API type-state**. C'est le « presque-faisable » à
opposer franchement au 71 vraiment infaisable : utile pour mesurer *où* exactement passe la ligne.

## Done
Binaire qui compile pour la cible ARM Linux et, **branché sur un Pi réel**, allume la LED / lit le capteur.
À défaut de matériel : compilation croisée verte + binding rppal généré, en notant explicitement que la
validation fonctionnelle reste suspendue au hardware.
