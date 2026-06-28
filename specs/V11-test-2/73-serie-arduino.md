# 73 — Communication série avec un Arduino
> **Domaine** Embarqué & matériel · **Crate(s)** `ironc add serialport` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Depuis un PC, dialoguer avec un Arduino sur le port USB-série : envoyer des commandes, lire les lignes
renvoyées (relevés de capteur, ACK…). Le programme tourne **sur l'hôte**, pas sur l'Arduino — c'est de
l'« embarqué » côté maître.

## Ce que ça teste dans Iron
`serialport` est un crate **`std` multiplateforme** : il ouvre `/dev/ttyUSB0` (ou `COMx`) et expose
`io::Read`/`io::Write`. Pas de `no_std`, pas d'`unsafe`, pas de cross-compile — l'un des rares « embarqué »
réellement dans le domaine d'Iron. Ce qu'il stresse :
- un **builder** : `serialport::new(port, baud).timeout(..).open()` ;
- un **trait-object** de retour `Box<dyn SerialPort>` — le binder doit le mapper proprement ;
- les **buffers `&mut [u8]`** read/write face à la sémantique par valeur (risque de clones inutiles),
  et la gestion de `io::ErrorKind::TimedOut` comme cas normal, pas comme erreur fatale.

## Approche
Ouvrir le port (baud, timeout), écrire une commande, lire la réponse octet par octet ou ligne par ligne,
décoder en UTF-8. L'Arduino exécute son propre sketch (hors périmètre Iron). Si le binder bute sur
`Box<dyn SerialPort>` ou le builder, un `.iri` à la main lève l'obstacle sans toucher au cœur Iron.

## Risque / limites attendues
🟡 — la seule vraie friction est le **builder + trait-object + buffers d'octets** ; rien d'infranchissable,
aucun mur d'invariant. Dépendance matérielle légère (un Arduino branché) pour le test bout-en-bout, mais la
compilation et toute la logique restent 100 % hôte.

## Done
Programme qui ouvre le port, échange au moins un aller-retour commande/réponse avec un Arduino réel — ou un
port série bouclé / émulé (`socat`) pour tester sans matériel — et décode correctement les lignes reçues.
