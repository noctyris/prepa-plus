# Prépa+

Application mobile et desktop pour consulter ses notes PRÉPA+ directement sur son téléphone, sans passer par le navigateur.

Fait avec [Dioxus](https://dioxuslabs.com) (Rust).

## Fonctionnalités

- Connexion à un compte PRÉPA+ depuis l'application
- Affichage des notes par semaine (matière, note, professeur, rang, moyenne, écart type)
- Session persistante : l'application se reconnecte automatiquement au lancement (identifiants stockés localement sur l'appareil)
- Disponible sur Android et desktop (Linux)

## Feuille de route

- [ ] Connexion à Pronote
- [ ] Style Material UI
- [ ] Widgets sur l'écran d'accueil
- [ ] Notifications quand une nouvelle note est publiée
- [ ] Gestion des erreurs (identifiants invalides, réseau indisponible, spinners…)
- [ ] Builds pour plusieurs architectures de téléphone
- [ ] Sauvegarde locale des notes en cas de problème de connexion
- [ ] Saisie manuelle des notes hors ligne, avec déduplication quand la note apparaît en ligne
- [ ] Planning des colles sauvegardé localement
- [x] Conservation des identifiants entre les lancements

## Construction

### Prérequis

- Rust (édition 2024)
- Pour Android : SDK Android, NDK, et la cible `aarch64-linux-android`

```bash
rustup target add aarch64-linux-android
cargo install dioxus-cli
```

### Desktop

```bash
dx serve
```

### Android (APK)

```bash
dx build --platform android --release --target aarch64-linux-android
```

L'APK est généré dans `target/dx/prepa-plus/release/android/app/build/outputs/apk/`.

## Sécurité

Les identifiants sont stockés dans le répertoire de données privé de l'application et ne sont jamais envoyés ailleurs qu'au serveur PRÉPA+.

## Licence

À définir.
