# Changelog

Notable changes to dindim are listed here. The format follows [Keep a Changelog](https://keepachangelog.com) and versions follow [Semantic Versioning](https://semver.org).

## [Unreleased]

## [0.1.0-alpha.1] (2026-11-20)

The first public preview. Expect rough edges, and please report what you find.

### Added

- Expenses, income and an emergency savings fund, each with its own color and icon.
- Quick entry built for the keyboard. Type an amount, a few letters of a category and a note, then press Enter. Amounts can be sums like `10 + 5.50`.
- Categories that belong to each account, created right from the entry form or managed in their own tab with usage counts and yearly totals.
- A monthly report in plain words, with spending by category, a comparison with the previous month, the savings balance and a chart of the whole year.
- Several accounts open at once, each with its own currency and an optional password that encrypts the file with SQLCipher.
- Undo and redo for transactions and categories.
- Search across every month by note, category or amount.
- CSV export and import.
- Backups to a single `.zip` file, plus automatic daily backups that keep the latest copies.
- Auto lock for password protected accounts after a period without use.
- Light and dark themes that can follow the system, adjustable zoom, and keyboard shortcuts for everything common.
- English and Brazilian Portuguese.
- Remembers the window size and position, the language, the theme and the last account opened.
- An optional check for new versions, off by default.
- Installers for Linux, macOS and Windows.

[Unreleased]: https://github.com/thdtr/dindim/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/thdtr/dindim/releases/tag/v0.1.0-alpha.1
