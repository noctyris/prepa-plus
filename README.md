> 🇫🇷 [Version française](https://github.com/noctyris/prepa-plus/blob/dev/README.fr.md)

# Prépa+

A mobile and desktop app for checking your PRÉPA+ grades straight from your phone, no browser needed.

Built with [Dioxus](https://dioxuslabs.com) (Rust).

## Features

- Log in to a PRÉPA+ account from within the app
- Grades displayed week by week (subject, grade, teacher, rank, average, standard deviation)
- Persistent session: the app logs back in automatically on launch (credentials stored locally on the device)
- Available on Android and desktop (Linux)

## Roadmap

- [ ] Pronote integration
- [ ] Material UI styling
- [ ] Home screen widgets
- [ ] Notifications when a new grade is published
- [ ] Error handling (invalid credentials, no network, spinners…)
- [ ] Builds for multiple phone architectures
- [ ] Offline caching of grades
- [ ] Manual grade entry while offline, with deduplication once the grade shows up online
- [ ] Offline *colles* (oral exams) schedule
- [x] Keep credentials between app launches

## Building

### Prerequisites

- Rust (2024 edition)
- For Android: Android SDK, NDK, and the `aarch64-linux-android` target

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

The APK is generated in `target/dx/prepa-plus/release/android/app/build/outputs/apk/`.

## Security

Credentials are stored in the app's private data directory and are never sent anywhere other than the PRÉPA+ server.

## License

To be determined.
