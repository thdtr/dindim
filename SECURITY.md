# Security

## Reporting a problem

Please don't open a public issue for security problems. Report them privately through [GitHub's advisory form](https://github.com/thdtr/dindim/security/advisories/new) or by email to **amandagimendes@duck.com**. You'll get an answer within a week, and we'll keep you posted until a fix ships.

Only the latest release gets security fixes.

## What a password protects

dindim works offline and never sends your data anywhere. When an account has a password, its file is encrypted with SQLCipher (AES-256, key derived with PBKDF2-HMAC-SHA512). Without the password, the file is unreadable, and there is no way to recover it if the password is lost.

A few things are worth knowing.

- Accounts without a password are stored as plain SQLite files. Anyone with access to your user folder can read them.
- When you add a password to an existing account, dindim writes a new encrypted file and replaces the old one. The operating system may keep pieces of the old unencrypted file on disk until they are overwritten, and snapshots or file history tools may keep full copies. For sensitive data, set the password when you create the account.
- Deleted accounts are moved to the `trash` folder inside your data folder, not erased. Delete that folder yourself if you want them gone for good.
- Backups keep each account exactly as it is on disk. Accounts with a password stay encrypted inside the backup; accounts without one don't.
- The account list (`accounts.json`) holds account names in plain text, so the start screen can show them before you unlock anything.
- Log files record what the app did and which errors happened. They never contain amounts, notes, category names or passwords.
- The update check is off by default. When you turn it on, dindim asks GitHub for the list of releases when it starts and sends nothing else.
