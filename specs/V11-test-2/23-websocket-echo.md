# 23 — Client WebSocket (echo)
> **Domaine** Réseau bas-niveau · **Crate(s)** `ironc add tungstenite` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Se connecter à un serveur WebSocket d'echo (`wss://…` public ou local), envoyer un message texte, le relire tel quel, enchaîner quelques allers-retours puis fermer proprement. Client **synchrone** (pas d'async).

## Ce que ça teste dans Iron
- Le binder face à une **struct générique** : `connect` rend un `WebSocket<MaybeTlsStream<TcpStream>>`. Iron doit exposer/inférer ce type paramétré sans que l'utilisateur le nomme (I15), ou via un alias posé dans le `.iri`.
- L'**enum `Message`** (`Text`/`Binary`/`Ping`/`Close`) : construction de variante typée + `match` au retour — pile dans les acquis V11 (construction de variante d'enum + scrutateur de `match` informant son type).
- La boucle `write_message` / `read_message` **bloquante** : un simple `while`, pas de boucle d'événements → compatible avec le modèle Iron.

## Approche
`connect(url)` → garder le socket, `write_message(Message.Text(...))`, `read_message()`, comparer, répéter, puis `Message.Close`. La couche TLS (`MaybeTlsStream`) tire `native-tls`/`rustls` via les *features* du crate, à activer dans le `Cargo.toml` généré.

## Risque / limites attendues
🟡 : pas de boucle d'événements (tungstenite est synchrone, ce qui **sauve** le projet), mais deux frottements : (1) le **type générique** du socket que le binder doit rendre utilisable, (2) les **features TLS** à activer côté `Cargo.toml`. L'enum `Message` passe bien. La variante *async* (`tokio-tungstenite`) serait 🔴 — on l'évite délibérément. Un `.iri` curé sur `connect`/`Message` lève le doute.

## Done
Le client envoie « hello », relit « hello », fait 3 allers-retours, ferme sans erreur ; le Rust généré compile et tourne contre un echo public ou local.
