# 25 — Client MQTT (publier / souscrire, IoT)
> **Domaine** Réseau bas-niveau · **Crate(s)** `ironc add rumqttc` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un client MQTT qui se connecte à un broker, **souscrit** à un topic et **publie** des messages, en affichant les messages entrants (cas IoT capteur/commande). Broker de test public (`test.mosquitto.org`) ou local.

## Ce que ça teste dans Iron
- Le modèle de rumqttc est une **boucle d'événements** : `Client::new` rend un **couple `(Client, Connection)`** ; il faut **piloter en continu** la `Connection` (`connection.iter()`, bloquant) qui **émet un flux d'`Event`/`Notification`**, pendant qu'un *handle* `Client` publie/souscrit en parallèle. C'est le motif réactif « driver + handle ».
- Les **enums d'événements profondément imbriqués** (`Event.Incoming(Packet.Publish{…})`, `QoS`) à `match` à chaque tour — stress du binder + du scrutateur de `match`.
- Le **partage cross-thread** : le thread qui *draine* la connexion mute l'état applicatif (messages reçus) que le thread de publication lit → **état mutable partagé** + closure de thread capturant cet état.

## Approche
`MqttOptions` (builder) → `Client::new` ; `subscribe(topic)` ; **un thread** qui boucle sur `connection.iter()` et `match` chaque `Event` entrant (mutation d'un état partagé `ref`) ; le thread principal `publish` périodiquement. La variante `AsyncClient` + `EventLoop.poll().await` = tokio (encore plus lourd).

## Risque / limites attendues
🔴 : **boucle d'événements + état mutable partagé + handle/driver scindés sur deux threads** = collision frontale avec « par valeur » et « pas d'état global mutable ». La consommation **réactive** (agir sur chaque event en mutant un état) est exactement ce à quoi le modèle Iron résiste : il faut un `ref` explicite (→ `Arc<Mutex>`) et une closure de thread qui le capture. Les **enums d'events imbriqués** taxent le binder. Atterrissage probable : `.iri` à la main sur `MqttOptions`/`Connection.iter` + `rust { }` pour la glue du driver. **Faisable** (rumqttc est à *messages/itérateur*, pas à *callbacks*) — d'où 🔴 et non « infaisable ».

## Done
Connexion au broker ; souscription à `iron/test` ; un `publish` sur ce topic est **reçu** et affiché par la boucle ; arrêt propre. Le Rust généré compile et échange réellement avec le broker.
