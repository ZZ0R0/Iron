# V10.1 — Inférence pilotée par les sites d'appel (cadrage)

> **Statut : CADRAGE (phase 1), en cours.** Raffinement du moteur V10 (pas un nouveau moteur). On cadre
> d'abord l'aspect central (R1) ; planification ensuite, sur feu vert.

## Pourquoi cette sous-version

V10 a livré un vrai moteur de contraintes, mais avec un **choix de conception discutable** : le
*let-polymorphism* instancie une **copie fraîche** de la signature à chaque appel → les sites d'appel
**n'informent pas** les vrais types de params. Conséquence (relevée à l'usage) : un param qu'aucune ligne du
corps ne détermine reste générique même s'il est **appelé** avec un type concret.

Or Iron compile le **programme entier** (inter-fonction, inter-fichier, inter-module ; les crates sont déjà
typées via `.iri`). **Tous les sites d'appel sont visibles.** Donc le type d'un param *est* déterminé par le
programme — V10 le jette. V10.1 le récupère.

## Périmètre (3 raffinements)

- **R1 — Inférence pilotée par les sites d'appel** *(le cœur, cadré ci-dessous)*. Par défaut, le type d'un
  param = **ce avec quoi on l'appelle** (monomorphe). La polymorphie n'apparaît **que** sur conflit.
- **R2 — Param « affichable » → générique `T: Display`** au lieu du défaut `str`. On porte la **borne** sur
  le générique implicite (les bornes sont déjà collectées) ; le `format!` Rust accepte tout `Display`. Garde :
  borne **numérique** → toujours `i64` (interaction avec les littéraux). *(petit, design tranché)*
- **R3 — Itération d'un param** : largement **subsumé par R1** (le site d'appel fixe `xs = list[…]`). Reliquat :
  un param itéré **sans aucun appel** (surface d'API) → annotation requise, comme une `pub fn` Rust typée.

## Cadrage de R1 — le modèle d'inférence

### Principe (ton modèle, raffiné)
- **Monomorphe par défaut** : les args d'un appel s'unifient avec les **vrais** params du callee (plus de
  copie fraîche systématique). `count([1,2,3])` ⇒ `xs = list[i64]`, sans annotation.
- **Générique seulement sur conflit** : si un param **non annoté** reçoit deux types **incompatibles** de
  deux sites d'appel, la fonction est **promue polymorphe** sur ce param (fresh-instanciée à chaque appel).
  `id(7)` + `id("hi")` ⇒ `id` devient `id[T]`. *(Voir l'arbitrage ci-dessous.)*

### Algorithme (point fixe, design retenu)
1. Hypothèse **monomorphe** : params = variables partagées (pas d'instanciation fraîche aux appels).
2. Génération whole-program ; les appels unifient args ↔ vrais params.
3. Solve. Un param qui unifierait deux types **incompatibles** → **conflit enregistré** (ni bind, ni erreur).
4. Param **non annoté** en conflit → marqué **générique** sur sa fonction (voir D-a). Param **annoté** en
   conflit → **vraie erreur de type** (D-b).
5. Re-génère/re-solve avec les params promus instanciés frais.
6. Répéter jusqu'au **point fixe** (la promotion ne fait que relâcher → termine).

### Décisions de design (tranchées par moi)
- **D-b ✅** Conflit sur un param **annoté** = erreur de type réelle (diag I8). On ne promeut jamais un type
  écrit par l'humain.
- **D-c ✅** Param **affichable seul** → générique `T: Display` (R2) ; **numérique** → `i64` ; **rien** →
  `T: Clone`.
- **D-d ✅** Itération/usage structurel d'un param → typé par le site d'appel (R1) ; pas d'appel → annoter.
- **D-e ✅** Granularité de promotion : **param entier** au départ (`id` → `id[T]`). L'**anti-unification**
  structurelle (`count` appelé en `list[i64]` ET `list[str]` → `list[T]`) est un **raffinement opt-in**,
  reporté tant qu'un besoin réel n'apparaît pas (cas rare).
- **D-f ✅** `rustc` reste le filet (I2) ; le point fixe ne bloque jamais un programme valide (best-effort +
  diagnostic strict des conflits primitifs, comme V10).

### Décision de fond — tranchée
- **D-a ✅ PROMOUVOIR EN GÉNÉRIQUE.** Un param **non annoté** appelé à deux types incompatibles → la fonction
  devient polymorphe sur ce param (`id` → `id[T]`). `id(7)` + `id("hi")` continuent de marcher, **zéro
  annotation**. *Justification perf (le critère utilisateur)* : Iron génère des **génériques Rust statiques**
  → **monomorphisation** par rustc (une copie spécialisée par type) → **coût runtime nul** (zero-cost ; pas de
  vtable/boxing/dispatch dynamique). Seul coût = compile-time/taille binaire, négligeable (peu de params sont
  réellement polymorphes). Les génériques **explicites** `def f[T]` existaient déjà (V3 → V9.1 4.1) ; le
  moteur ne fait qu'en **inventer** quand c'est justifié.

### Décisions complémentaires (cadrage exhaustif — points qui restaient implicites)

- **D-g ✅ (le gros) — La boucle de point fixe re-exécute GÉNÉRATION + solve, pas seulement solve.** C'est la
  formalisation de « résoudre graduellement » : à chaque tour, les types fraîchement résolus (un param fixé
  par son site d'appel) sont **réinjectés** dans la génération → les **usages structurels** d'un param se
  résolvent alors (`x.name` une fois `x = Point` connu ; `obj.m()` une fois le receveur connu ; `xs[i]`).
  Sans ça, un retour déterminé par `return x.name` (param non annoté, type venu de l'appel) resterait flou.
  Termine : chaque tour **lie strictement plus** de variables **ou** promeut un param (bornes finies).
- **D-h ✅ — Param d'ordre supérieur** (un param **appelé** comme fonction, `f(x)`) : type `fn(…) -> …`
  déterminé par le site d'appel (quand on passe une fonction concrète). Pas d'appel concret → repli
  heuristique `fn(i64) -> i64` (comme V9), sinon annoter.
- **D-i ✅ — Récursion (simple et mutuelle)** : gérée nativement par la passe **whole-program** (tout dans un
  seul jeu de contraintes ; les appels récursifs unifient avec les vrais params partagés). **Pas** de tri par
  SCC nécessaire ; la promotion s'applique après convergence, uniformément.
- **D-j ✅ — Le defaulting est la DERNIÈRE étape**, après convergence du point fixe (numérique→`i64`,
  `Display`→`str`, sinon générique). Jamais avant — sinon on fige un type trop tôt (le bug « str dans `+` »).
- **D-k ✅ — Signatures externes génériques** (`.iri`, ex. `Vec::push<T>`) : **hors-périmètre** (cohérent avec
  la couverture binding 62–92 %, le « 0.1 % ») — traitées opaques/concrètes, `rustc` filet.
- **D-l ✅ — Généralisation des retours** : un retour qui dépend d'un param générique est lui-même générique
  (`make_pair(a, b) -> (A, B)`) — automatique via la réécriture (le retour résolu porte les génériques).
- **D-f (rappel) — Promotion best-effort** : un param promu qui hériterait de bornes incompatibles (à la fois
  numérique **et** itéré) peut produire un générique rejeté par `rustc` → filet, pas de résolution exotique.

## Impact invariants
- **Raffine I15** (inférence par contraintes) : ajoute « pilotée par les sites d'appel, polymorphie sur
  conflit ». I2/I6 inchangés. `rustc` filet conservé.

## Critère e2e (gate V10.1, à planifier ensuite)
Corpus 50 + binding 30 + vitrine + zero-annot **toujours verts** ; **plus** : `count(xs)` & co. (param typé
par l'appel) compilent **sans annotation** ; `id` reste polymorphe (si D-a = promotion) ; R2 (`label`) marche.

> **Sortie de cadrage** : **D-a…D-l toutes tranchées.** Plus aucune décision structurante implicite dans
> cette catégorie (modèle d'inférence, algorithme, incrémentalité, récursion, HOF, defaulting, externes,
> retours). R2/R3 bornés. **Cadrage R1 = 100 %.** Prêt à planifier sur feu vert.
