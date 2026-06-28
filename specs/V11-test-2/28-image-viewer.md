# 28 — Visionneuse d'images
> **Domaine** GUI desktop · **Crate(s)** `ironc add iced` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Ouvrir une image, l'afficher, naviguer (précédent/suivant) dans un dossier. Le **décodage** (crate `image`) est trivial en Iron ; la coquille **iced** est le mur — et iced stresse Iron *différemment* d'egui.

## Ce que ça teste dans Iron
iced suit l'**architecture Elm** (retenue/réactive), encore plus chargée en builders/génériques qu'egui :
- **Boucle d'événements possédée par le runtime iced** : tu ne « tournes » pas la boucle, tu y **réagis**.
- **Dispatch par `Message`** : `impl Application` avec un **enum `Message`** + `update(&mut self, msg) -> Command` — le runtime t'appelle avec des messages, tu mutes `&mut self`. **Types associés** (`Message`, `Theme`, `Executor`, `Flags`) à fournir.
- **Arbre de vue par builders** : `view(&self) -> Element<Message>` se compose en **chaînes de combinateurs** (`column![…]`, `button(…).on_press(Message.X)`) — builders = point faible binder, et `.on_press` **câble un callback producteur de message**.
- **Générique + lifetime** : `Element<'a, Message, Renderer>` — stress direct de l'inférence (I15) et des lifetimes, hors-périmètre côté inférence pure.

## Approche (honnête)
`impl Application` + `Message` + `view` en **`rust { }`** quasi certainement ; le décodage/chargement (`image::open`, dimensions, octets) s'écrit en Iron. On mesure combien de la coquille Elm le binder peut exposer avant l'interop manuelle.

## Risque / limites attendues
🔴 : **boucle Elm + dispatch `Message` + arbre de vue par builders + types associés + lifetimes sur `Element`** s'empilent. Le `&mut self` muté par le runtime au fil des messages est l'**état réactif possédé par le framework**, que « par valeur / pas d'état global mutable » n'accueille pas. Les **builders + génériques à lifetime** sont précisément ce que le binder ne garantit pas (couverture auto mesurée, jamais 100 %). Verdict : coquille en `rust { }`.

## Done
Ouvrir un PNG l'affiche à l'échelle ; « suivant » passe à l'image d'après ; build vert. Part Iron (décodage) vs part `rust { }` (coquille iced) **documentée**.
