//! Backup archives: one `.zip` with every account file and a small manifest.
//! Password protected accounts stay encrypted inside the archive.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;

use crate::error::BackupProblem;
use crate::ledger::{SQLITE_HEADER, sync_parent_dir};
use crate::schema::CURRENT_VERSION;
use crate::vault::{EXTENSION, ManifestEntry, is_valid_id};
use crate::{Result, StoreError};

const MANIFEST: &str = "dindim-backup.json";
const FORMAT: u32 = 1;
const MAX_ACCOUNT_BYTES: u64 = 1 << 30;
const MAX_MANIFEST_BYTES: u64 = 1 << 20;
const FILE_PREFIX: &str = "dindim-backup-";

#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    format: u32,
    app_version: String,
    accounts: Vec<ManifestEntry>,
}

/// Account files copied into memory, ready to be written somewhere else.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub(crate) accounts: Vec<SnapshotAccount>,
}

#[derive(Debug, Clone)]
pub(crate) struct SnapshotAccount {
    pub id: String,
    pub name: String,
    pub bytes: Vec<u8>,
}

impl Snapshot {
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// Writes the archive to `dest` through a temporary file, so a failure
    /// never leaves half a backup behind.
    pub fn write_zip(&self, dest: &Path) -> Result<()> {
        let tmp = dest.with_extension("partial");
        let result = self.write_zip_to(&tmp).and_then(|()| {
            fs::rename(&tmp, dest)?;
            sync_parent_dir(dest);
            Ok(())
        });
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result
    }

    fn write_zip_to(&self, path: &Path) -> Result<()> {
        let mut zip = zip::ZipWriter::new(fs::File::create(path)?);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let manifest = BackupManifest {
            format: FORMAT,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            accounts: self
                .accounts
                .iter()
                .map(|a| ManifestEntry {
                    id: a.id.clone(),
                    name: a.name.clone(),
                })
                .collect(),
        };
        zip.start_file(MANIFEST, options)?;
        let text = serde_json::to_string_pretty(&manifest).map_err(std::io::Error::other)?;
        zip.write_all(text.as_bytes())?;
        for account in &self.accounts {
            zip.start_file(entry_name(&account.id), options)?;
            zip.write_all(&account.bytes)?;
        }
        zip.finish()?.sync_all()?;
        Ok(())
    }
}

fn entry_name(id: &str) -> String {
    format!("accounts/{id}.{EXTENSION}")
}

/// Accounts read and checked from a backup, not yet added to any vault.
#[derive(Debug)]
pub struct BackupContents {
    pub(crate) accounts: Vec<SnapshotAccount>,
}

impl BackupContents {
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
}

/// Reads a backup and checks that every account file looks like a database
/// this version can open.
pub fn read(src: &Path) -> Result<BackupContents> {
    let not_a_backup = || StoreError::InvalidBackup(BackupProblem::NotABackup);
    let mut zip = zip::ZipArchive::new(fs::File::open(src)?).map_err(|e| match e {
        zip::result::ZipError::Io(io) => StoreError::Io(io),
        _ => not_a_backup(),
    })?;
    let manifest: BackupManifest = {
        let file = zip.by_name(MANIFEST).map_err(|_| not_a_backup())?;
        let mut text = String::new();
        file.take(MAX_MANIFEST_BYTES).read_to_string(&mut text)?;
        serde_json::from_str(&text).map_err(|_| not_a_backup())?
    };
    if manifest.format > FORMAT {
        return Err(StoreError::TooNew);
    }

    let mut accounts = Vec::new();
    for entry in manifest.accounts {
        if !is_valid_id(&entry.id) {
            return Err(StoreError::InvalidBackup(BackupProblem::Damaged));
        }
        let file = zip.by_name(&entry_name(&entry.id))?;
        let mut bytes = Vec::new();
        file.take(MAX_ACCOUNT_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_ACCOUNT_BYTES {
            return Err(StoreError::InvalidBackup(BackupProblem::TooLarge));
        }
        check_database(&bytes)?;
        accounts.push(SnapshotAccount {
            id: entry.id,
            name: entry.name,
            bytes,
        });
    }
    Ok(BackupContents { accounts })
}

/// Plain files must open and pass an integrity check. Encrypted files can't
/// be read without the password, so only their shape is checked: SQLCipher
/// pages come in multiples of 512 bytes.
fn check_database(bytes: &[u8]) -> Result<()> {
    let bad = || StoreError::InvalidBackup(BackupProblem::NotADatabase);
    if !bytes.starts_with(SQLITE_HEADER) {
        return if bytes.len() >= 1024 && bytes.len().is_multiple_of(512) {
            Ok(())
        } else {
            Err(bad())
        };
    }
    let dir = tempdir()?;
    let path = dir.join("check.db");
    fs::write(&path, bytes)?;
    let outcome = (|| -> rusqlite::Result<(String, i64)> {
        let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let check: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        Ok((check, version))
    })();
    let _ = fs::remove_dir_all(&dir);
    match outcome {
        Ok((check, version)) if check == "ok" && version <= CURRENT_VERSION => Ok(()),
        Ok((check, _)) if check == "ok" => Err(StoreError::TooNew),
        _ => Err(bad()),
    }
}

fn tempdir() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for attempt in 0..100u32 {
        let dir = base.join(format!(
            "dindim-check-{}-{}-{attempt}",
            std::process::id(),
            jiff::Timestamp::now().as_nanosecond()
        ));
        match fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(std::io::Error::other("could not create a temporary directory").into())
}

/// File name for a backup taken now, like `dindim-backup-2026-11-20-143005.zip`.
pub fn file_name(now: jiff::civil::DateTime) -> String {
    format!("{FILE_PREFIX}{}.zip", now.strftime("%Y-%m-%d-%H%M%S"))
}

/// Backups in `dir` made by dindim, newest first.
pub fn list(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(items) => items
            .filter_map(|item| item.ok().map(|i| i.path()))
            .filter(|p| {
                p.extension().is_some_and(|e| e == "zip")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(FILE_PREFIX))
            })
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e.into()),
    };
    found.sort();
    found.reverse();
    Ok(found)
}

/// Deletes the oldest backups in `dir`, keeping the newest `keep`.
pub fn prune(dir: &Path, keep: usize) -> Result<usize> {
    let mut removed = 0;
    for old in list(dir)?.into_iter().skip(keep) {
        fs::remove_file(&old)?;
        removed += 1;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vault;
    use crate::vault::tests::entry;
    use dindim_core::Currency;

    fn vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let (v, _) = Vault::open(dir.path().join("data")).unwrap();
        (dir, v)
    }

    #[test]
    fn round_trip_keeps_data_and_passwords() {
        let (_d, source) = vault();
        let (_, mut l) = source.create_account("Joe", None, Currency::USD).unwrap();
        l.add_entry(&entry()).unwrap();
        drop(l);
        let (_, mut l) = source
            .create_account("Ann", Some("secret"), Currency::USD)
            .unwrap();
        l.add_entry(&entry()).unwrap();
        l.add_entry(&entry()).unwrap();
        drop(l);

        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("backup.zip");
        let snapshot = source.snapshot().unwrap();
        assert_eq!(snapshot.len(), 2);
        snapshot.write_zip(&file).unwrap();
        assert!(!file.with_extension("partial").exists());

        let (_d2, target) = vault();
        target.create_account("Joe", None, Currency::USD).unwrap();
        let imported = target.install(read(&file).unwrap()).unwrap();
        let names: Vec<_> = imported.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Joe (2)", "Ann"]);
        assert!(imported[1].protected);

        let joe = target.unlock(&imported[0].id, None).unwrap();
        assert_eq!(joe.entries().unwrap().len(), 1);
        assert!(target.unlock(&imported[1].id, None).is_err());
        let ann = target.unlock(&imported[1].id, Some("secret")).unwrap();
        assert_eq!(ann.entries().unwrap().len(), 2);
        assert_eq!(target.accounts().unwrap().len(), 3);
    }

    fn zip_with(path: &Path, files: &[(&str, &[u8])]) {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
        for (name, bytes) in files {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }

    fn problem(path: &Path) -> Option<BackupProblem> {
        match read(path) {
            Err(StoreError::InvalidBackup(p)) => Some(p),
            _ => None,
        }
    }

    #[test]
    fn rejects_files_that_are_not_backups() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("fake.zip");
        fs::write(&fake, "not a zip").unwrap();
        assert_eq!(problem(&fake), Some(BackupProblem::NotABackup));

        let foreign = dir.path().join("foreign.zip");
        zip_with(&foreign, &[("notes.txt", b"hello")]);
        assert_eq!(problem(&foreign), Some(BackupProblem::NotABackup));
    }

    #[test]
    fn rejects_accounts_that_are_not_databases() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = br#"{"format":1,"app_version":"0","accounts":[{"id":"joe","name":"Joe"}]}"#;

        let garbage = dir.path().join("garbage.zip");
        zip_with(
            &garbage,
            &[(MANIFEST, manifest), ("accounts/joe.dindim", b"just text")],
        );
        assert_eq!(problem(&garbage), Some(BackupProblem::NotADatabase));

        let missing = dir.path().join("missing.zip");
        zip_with(&missing, &[(MANIFEST, manifest)]);
        assert_eq!(problem(&missing), Some(BackupProblem::MissingAccount));

        let escape = dir.path().join("escape.zip");
        zip_with(
            &escape,
            &[(
                MANIFEST,
                br#"{"format":1,"app_version":"0","accounts":[{"id":"../x","name":"X"}]}"#,
            )],
        );
        assert_eq!(problem(&escape), Some(BackupProblem::Damaged));
    }

    #[test]
    fn refuses_backups_from_newer_versions() {
        let dir = tempfile::tempdir().unwrap();
        let newer = dir.path().join("newer.zip");
        zip_with(
            &newer,
            &[(
                MANIFEST,
                br#"{"format":99,"app_version":"9","accounts":[]}"#,
            )],
        );
        assert!(matches!(read(&newer), Err(StoreError::TooNew)));
    }

    #[test]
    fn prunes_old_backups() {
        let dir = tempfile::tempdir().unwrap();
        for day in 1..=5 {
            fs::write(
                dir.path()
                    .join(format!("{FILE_PREFIX}2026-11-0{day}-120000.zip")),
                "",
            )
            .unwrap();
        }
        fs::write(dir.path().join("other.zip"), "").unwrap();
        assert_eq!(prune(dir.path(), 3).unwrap(), 2);
        let left = list(dir.path()).unwrap();
        assert_eq!(left.len(), 3);
        assert!(left[0].ends_with(format!("{FILE_PREFIX}2026-11-05-120000.zip")));
        assert!(dir.path().join("other.zip").exists());
    }
}
