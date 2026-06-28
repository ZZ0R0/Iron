# 75 — Lecture d'un capteur I²C/SPI (Linux)
> **Domaine** Embarqué & matériel · **Crate(s)** `ironc add linux-embedded-hal` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Sous Linux (Pi, BeagleBone, tout SBC), lire un capteur sur le bus I²C ou SPI — température, pression,
accéléromètre… — via `/dev/i2c-N` ou `/dev/spidev`. Toujours côté Linux, pas de firmware.

## Ce que ça teste dans Iron
`linux-embedded-hal` est **`std`** (pas de `no_std`/`unsafe` côté utilisateur), mais il est la **porte vers
l'écosystème `embedded-hal`**, qui est **profondément générique** :
- les crates de drivers (ex. `bmp280`, `mpu6050`) sont **génériques sur les traits**
  `embedded_hal::i2c::I2c` / `SpiDevice` / `DelayNs` ;
- composer un driver générique avec l'impl concrète `linux-embedded-hal` = une **instanciation de
  génériques + types associés + bornes de traits** que le binder V8 ne sait pas exprimer.
C'est le test de la généricité HAL, là où l'auto-binding casse net.

## Approche
**Contourner les drivers génériques** : utiliser directement `I2cdev::new("/dev/i2c-1")` et ses
transactions brutes `write` / `read` / `write_read` d'octets, puis **décoder les registres du capteur à la
main** (datasheet). On perd l'écosystème de drivers réutilisables, mais on reste sur une API concrète,
mappable par le binder. Matériel requis pour valider.

## Risque / limites attendues
🔴 — le mur n'est pas l'`unsafe` mais la **généricité des traits `embedded-hal`** : tout l'intérêt de
l'écosystème (drivers partagés entre plateformes) est hors de portée du binder. Faisable **seulement** au
niveau octet brut `/dev/i2c`, au prix de réécrire le décodage capteur à la main.

## Done
Programme qui ouvre le bus I²C/SPI, lit les registres d'un capteur réel et imprime une mesure cohérente
(ex. température en °C), en restant sur l'API brute sans driver générique — et en notant l'écosystème perdu.
