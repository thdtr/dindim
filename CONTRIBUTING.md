# Contributing to dindim

Thanks for taking the time to help. Bug reports, translations, ideas and code are all welcome.

## Before you start

For anything bigger than a small fix, open an issue first and describe what you have in mind. It saves everyone from work that ends up going in a different direction.

By sending a pull request you agree that your contribution is licensed under the [Apache License 2.0](LICENSE), like the rest of the project.

## Setting up

You need a recent stable [Rust](https://rustup.rs) toolchain (1.92 or newer) and Perl, which the build uses to compile the bundled OpenSSL for SQLCipher. Perl ships with macOS and most Linux distributions. On Windows, [Strawberry Perl](https://strawberryperl.com) works.

```bash
git clone https://github.com/thdtr/dindim
cd dindim
cargo run -p dindim
```

To try things without touching your real data, point the app at a scratch folder.

```bash
DINDIM_HOME=/tmp/dindim-dev cargo run -p dindim
```

`DINDIM_LOG=debug` turns on detailed logs.

## Checks

Pull requests need these to pass, and CI runs them on Linux, macOS and Windows.

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## How the code is organized

The workspace has four crates, and dependencies only point one way.

| Crate | What it does |
| --- | --- |
| `dindim-core` | Money, entries, categories, months and reports. Pure logic with no I/O and no user-facing text. |
| `dindim-store` | Account files (SQLite, encrypted with SQLCipher when there's a password), the account folder, backups and CSV. |
| `dindim-i18n` | Translations in [Fluent](https://projectfluent.org) and locale-aware formatting of money and dates. |
| `dindim` | The desktop app, built with [iced](https://iced.rs) in the Elm style. State and messages live in `app/`, screens in `views/`. Screens only draw; they never change state. |

Database changes go in a new migration at the end of the list in `crates/dindim-store/src/schema.rs`. Released migrations are never edited.

## Translations

All text people see lives in `crates/dindim-i18n/locales/<language>/main.ftl`. A test fails if a language is missing a message, and another fails if the code asks for a message that doesn't exist.

When you write or translate text, write it the way people who speak that language actually talk. A literal translation usually reads as one. Keep it short and friendly, and avoid jargon.

To add a language, copy the `en-US` folder, translate it, and add the language to `Locale` in `crates/dindim-i18n/src/lib.rs`.

## Screenshots

The README screenshots are taken on a real macOS desktop by the Screenshots workflow. Run it from the Actions tab, download the `screenshots` artifact and copy the images into `docs/`. Then rebuild the social preview card.

```bash
python3 tools/make_social_preview.py
```

To try the app with the same demo data, create a demo account first.

```bash
cargo run -p dindim-store --example demo -- /tmp/dindim-demo/data tools/screenshot/demo/en-US.csv Personal USD
DINDIM_HOME=/tmp/dindim-demo cargo run -p dindim
```

## Releases

Versions follow [Semantic Versioning](https://semver.org). Until 1.0, releases are tagged as alpha, beta or release candidate, like `v0.1.0-alpha.1`. Pushing a tag builds every platform, attaches checksums and provenance attestations, and publishes the release with the matching section of the [changelog](CHANGELOG.md).
