# 22 — Scanner de ports / ping UDP
> **Domaine** Réseau bas-niveau · **Crate(s)** `import std` — `net`, `time` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Sonder une plage de ports TCP d'un hôte (`connect_timeout`) et signaler ceux qui sont ouverts ; en bonus, un aller-retour **UDP** (envoi d'un datagramme + attente d'une réponse, type echo). Sortie : liste des ports ouverts + latence mesurée.

## Ce que ça teste dans Iron
- `net` en lecture seule de bout en bout : `TcpStream.connect_timeout`, `UdpSocket.bind`/`send_to`/`recv_from`, parsing d'adresse (`"1.2.3.4:80".parse()` → `Result`), `Duration` (`std.time`).
- **L'erreur-comme-valeur** : un port fermé renvoie `Err` (connexion refusée) ; c'est le cas-école de l'inférence `Result`/`Option` d'Iron (I11) et du sucre `try`/`except` → `match`. Un scan = boucler en absorbant les `Err` sans paniquer.
- Boucles + accumulation dans une `list` (compréhension possible) : primitives de langage pures, zéro crate.

## Approche
Boucle sur la plage de ports, `connect_timeout` court, classer ouvert/fermé selon `Ok`/`Err`. Optionnel : paralléliser avec un pool de `thread.spawn` pour la vitesse. Côté UDP, `bind` local, `send_to`, `recv_from` avec `set_read_timeout` pour borner l'attente.

## Risque / limites attendues
🟢 : API std « feuille », pas de closure, pas d'état partagé, pas de boucle d'événements. Seule **honnêteté** à poser : un **vrai `ping` ICMP** exige des *raw sockets* (privilèges root + souvent `libc`/`unsafe`) — **hors d'atteinte** pour Iron (jamais d'`unsafe`, I2). On reste donc sur **TCP-connect** et **datagrammes UDP**, ce qui couvre l'usage sans jamais toucher au mur.

## Done
`scanner 127.0.0.1 1-1024` liste les ports ouverts (p. ex. 22, 80) ; un port fermé n'interrompt pas le scan ; l'aller-retour UDP affiche une latence. Build vert sur le Rust généré.
