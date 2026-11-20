//! The folder that holds every account.
//!
//! ```text
//! <root>/
//! ├── accounts.json          account names and order
//! ├── accounts/<id>.dindim   one SQLite (or SQLCipher) file per account
//! ├── backups/               automatic backups
//! └── trash/                 removed accounts, never destroyed
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use dindim_core::Currency;
use dindim_core::text::{fold, slugify};
use serde::{Deserialize, Serialize};

use crate::backup::{BackupContents, Snapshot, SnapshotAccount};
use crate::ledger::{Ledger, clean_name, is_encrypted, sync_file, sync_parent_dir};
use crate::{Result, StoreError};

const MANIFEST: &str = "accounts.json";
const ACCOUNTS_DIR: &str = "accounts";
const BACKUPS_DIR: &str = "backups";
const TRASH_DIR: &str = "trash";
pub(crate) const EXTENSION: &str = "dindim";

/// What the start screen needs to know about an account without opening it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub protected: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Manifest {
    #[serde(default)]
    accounts: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ManifestEntry {
    pub id: String,
    pub name: String,
}

/// Account ids double as file names, so only slugs are accepted.
pub(crate) fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[derive(Debug, Clone)]
pub struct Vault {
    root: PathBuf,
}

impl Vault {
    /// Opens (creating if needed) the vault at `root`. When the account list
    /// is unreadable it is moved aside and the path of that copy is returned,
    /// so nothing is overwritten silently.
    pub fn open(root: impl Into<PathBuf>) -> Result<(Self, Option<PathBuf>)> {
        let vault = Vault { root: root.into() };
        fs::create_dir_all(vault.accounts_dir())?;
        let damaged = match vault.read_manifest() {
            Ok(_) => None,
            Err(ManifestError::Io(e)) => return Err(e.into()),
            Err(ManifestError::Damaged) => Some(vault.quarantine_manifest()?),
        };
        Ok((vault, damaged))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn accounts_dir(&self) -> PathBuf {
        self.root.join(ACCOUNTS_DIR)
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.root.join(BACKUPS_DIR)
    }

    fn path_of(&self, id: &str) -> Result<PathBuf> {
        if !is_valid_id(id) {
            return Err(StoreError::AccountNotFound);
        }
        Ok(self.accounts_dir().join(format!("{id}.{EXTENSION}")))
    }

    fn read_manifest(&self) -> Result<Manifest, ManifestError> {
        match fs::read_to_string(self.root.join(MANIFEST)) {
            Ok(text) => serde_json::from_str(&text).map_err(|_| ManifestError::Damaged),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Manifest::default()),
            Err(e) => Err(ManifestError::Io(e)),
        }
    }

    fn load_manifest(&self) -> Result<Manifest> {
        match self.read_manifest() {
            Ok(manifest) => Ok(manifest),
            Err(ManifestError::Io(e)) => Err(e.into()),
            Err(ManifestError::Damaged) => {
                self.quarantine_manifest()?;
                Ok(Manifest::default())
            }
        }
    }

    fn quarantine_manifest(&self) -> Result<PathBuf> {
        let stamp = jiff::Timestamp::now().strftime("%Y%m%dT%H%M%SZ");
        let target = self.root.join(format!("{MANIFEST}.damaged-{stamp}"));
        fs::rename(self.root.join(MANIFEST), &target)?;
        tracing::warn!(path = %target.display(), "account list was unreadable and was set aside");
        Ok(target)
    }

    fn save_manifest(&self, manifest: &Manifest) -> Result<()> {
        let path = self.root.join(MANIFEST);
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(manifest).map_err(std::io::Error::other)?;
        fs::write(&tmp, text)?;
        sync_file(&tmp)?;
        fs::rename(&tmp, &path)?;
        sync_parent_dir(&path);
        Ok(())
    }

    /// Accounts in the order they were added. Files dropped into the folder
    /// by hand show up too, named after the file.
    pub fn accounts(&self) -> Result<Vec<AccountInfo>> {
        let manifest = self.load_manifest()?;
        let mut out = Vec::new();
        for entry in manifest.accounts.iter().filter(|e| is_valid_id(&e.id)) {
            let path = self.path_of(&entry.id)?;
            if path.exists() && !out.iter().any(|a: &AccountInfo| a.id == entry.id) {
                out.push(AccountInfo {
                    id: entry.id.clone(),
                    name: entry.name.clone(),
                    protected: is_encrypted(&path)?,
                });
            }
        }
        let mut orphans = Vec::new();
        for item in fs::read_dir(self.accounts_dir())? {
            let path = item?.path();
            if path.extension().and_then(|e| e.to_str()) != Some(EXTENSION) {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if is_valid_id(id) && !out.iter().any(|a| a.id == id) {
                orphans.push(AccountInfo {
                    id: id.to_string(),
                    name: id.to_string(),
                    protected: is_encrypted(&path)?,
                });
            }
        }
        orphans.sort_by(|a, b| a.id.cmp(&b.id));
        out.extend(orphans);
        Ok(out)
    }

    fn register(&self, entries: &[ManifestEntry]) -> Result<()> {
        let mut manifest = self.load_manifest()?;
        for entry in entries {
            manifest.accounts.retain(|a| a.id != entry.id);
            manifest.accounts.push(entry.clone());
        }
        self.save_manifest(&manifest)
    }

    fn ensure_unique_name(&self, name: &str, except: Option<&str>) -> Result<()> {
        let wanted = fold(name);
        let taken = self
            .accounts()?
            .iter()
            .any(|a| fold(&a.name) == wanted && Some(a.id.as_str()) != except);
        if taken {
            return Err(StoreError::NameTaken);
        }
        Ok(())
    }

    fn unique_id(&self, name: &str, reserved: &[String]) -> Result<String> {
        let base = Some(slugify(name))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "account".into());
        let base: String = base.chars().take(60).collect();
        let mut id = base.clone();
        let mut n = 2;
        while self.path_of(&id)?.exists() || reserved.contains(&id) {
            id = format!("{base}-{n}");
            n += 1;
        }
        Ok(id)
    }

    /// A free name based on `name`: "Joe", then "Joe (2)", "Joe (3)"...
    fn available_name(&self, name: &str, reserved: &[String]) -> Result<String> {
        let mut taken: Vec<String> = self.accounts()?.iter().map(|a| fold(&a.name)).collect();
        taken.extend(reserved.iter().map(|n| fold(n)));
        let mut candidate = name.to_string();
        let mut n = 2;
        while taken.contains(&fold(&candidate)) {
            candidate = format!("{name} ({n})");
            n += 1;
        }
        Ok(candidate)
    }

    pub fn create_account(
        &self,
        name: &str,
        password: Option<&str>,
        currency: Currency,
    ) -> Result<(AccountInfo, Ledger)> {
        let name = clean_name(name)?;
        self.ensure_unique_name(&name, None)?;
        let id = self.unique_id(&name, &[])?;
        let ledger = Ledger::create(&self.path_of(&id)?, password, currency)?;
        self.register(&[ManifestEntry {
            id: id.clone(),
            name: name.clone(),
        }])?;
        tracing::info!(account = %id, "account created");
        let info = AccountInfo {
            id,
            name,
            protected: password.is_some_and(|p| !p.is_empty()),
        };
        Ok((info, ledger))
    }

    pub fn unlock(&self, id: &str, password: Option<&str>) -> Result<Ledger> {
        Ledger::open(&self.path_of(id)?, password)
    }

    pub fn rename_account(&self, id: &str, new_name: &str) -> Result<String> {
        let name = clean_name(new_name)?;
        if !self.path_of(id)?.exists() {
            return Err(StoreError::AccountNotFound);
        }
        self.ensure_unique_name(&name, Some(id))?;
        let mut manifest = self.load_manifest()?;
        match manifest.accounts.iter_mut().find(|a| a.id == id) {
            Some(entry) => entry.name = name.clone(),
            None => manifest.accounts.push(ManifestEntry {
                id: id.to_string(),
                name: name.clone(),
            }),
        }
        self.save_manifest(&manifest)?;
        Ok(name)
    }

    /// Moves the account file to the trash folder and returns its new path.
    pub fn remove_account(&self, id: &str) -> Result<PathBuf> {
        let path = self.path_of(id)?;
        if !path.exists() {
            return Err(StoreError::AccountNotFound);
        }
        let trash = self.root.join(TRASH_DIR);
        fs::create_dir_all(&trash)?;
        let stamp = jiff::Timestamp::now().strftime("%Y%m%dT%H%M%SZ");
        let target = trash.join(format!("{id}-{stamp}.{EXTENSION}"));
        fs::rename(&path, &target)?;
        let mut manifest = self.load_manifest()?;
        manifest.accounts.retain(|a| a.id != id);
        self.save_manifest(&manifest)?;
        tracing::info!(account = %id, "account moved to trash");
        Ok(target)
    }

    /// Copies every account file into memory.
    ///
    /// The app writes to ledgers only from its main thread and never keeps a
    /// transaction open between calls, so the files are consistent whenever
    /// this runs on that same thread.
    pub fn snapshot(&self) -> Result<Snapshot> {
        let accounts = self
            .accounts()?
            .into_iter()
            .map(|a| {
                Ok(SnapshotAccount {
                    bytes: fs::read(self.path_of(&a.id)?)?,
                    id: a.id,
                    name: a.name,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Snapshot { accounts })
    }

    /// Adds the accounts from a backup. Nothing is overwritten: names already
    /// in use get a number, like "Joe (2)". Files are staged first and only
    /// then moved in, so a failure leaves the vault as it was.
    pub fn install(&self, contents: BackupContents) -> Result<Vec<AccountInfo>> {
        let mut planned: Vec<(ManifestEntry, PathBuf, Vec<u8>)> = Vec::new();
        for account in contents.accounts {
            let reserved_names: Vec<String> = planned.iter().map(|p| p.0.name.clone()).collect();
            let reserved_ids: Vec<String> = planned.iter().map(|p| p.0.id.clone()).collect();
            let base = clean_name(&account.name).unwrap_or_else(|_| "Account".into());
            let name = self.available_name(&base, &reserved_names)?;
            let id = self.unique_id(&name, &reserved_ids)?;
            let path = self.path_of(&id)?;
            planned.push((ManifestEntry { id, name }, path, account.bytes));
        }

        let mut staged = Vec::new();
        let result = (|| -> Result<()> {
            for (_, path, bytes) in &planned {
                let tmp = path.with_extension("importing");
                fs::write(&tmp, bytes)?;
                sync_file(&tmp)?;
                staged.push((tmp, path.clone()));
            }
            Ok(())
        })();
        if let Err(e) = result {
            for (tmp, _) in &staged {
                let _ = fs::remove_file(tmp);
            }
            return Err(e);
        }
        for (tmp, path) in &staged {
            fs::rename(tmp, path)?;
        }
        let entries: Vec<ManifestEntry> = planned.iter().map(|p| p.0.clone()).collect();
        self.register(&entries)?;
        tracing::info!(accounts = entries.len(), "backup imported");

        planned
            .into_iter()
            .map(|(entry, path, _)| {
                Ok(AccountInfo {
                    protected: is_encrypted(&path)?,
                    id: entry.id,
                    name: entry.name,
                })
            })
            .collect()
    }
}

enum ManifestError {
    Io(std::io::Error),
    Damaged,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use dindim_core::{Date, Kind, Money, NewEntry};

    fn vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let (v, damaged) = Vault::open(dir.path().join("data")).unwrap();
        assert!(damaged.is_none());
        (dir, v)
    }

    pub(crate) fn entry() -> NewEntry {
        NewEntry {
            date: Date::constant(2026, 11, 19),
            kind: Kind::Expense,
            amount: Money::from_minor(500),
            category_id: None,
            note: "coffee".into(),
        }
    }

    #[test]
    fn creates_lists_and_unlocks_accounts() {
        let (_d, v) = vault();
        assert!(v.accounts().unwrap().is_empty());
        let (joe, mut ledger) = v.create_account("Joe", None, Currency::USD).unwrap();
        ledger.add_entry(&entry()).unwrap();
        drop(ledger);
        let (zoe, _) = v
            .create_account("Zoë", Some("1234"), Currency::EUR)
            .unwrap();
        assert_eq!(zoe.id, "zoe");

        let list = v.accounts().unwrap();
        assert_eq!(list, vec![joe.clone(), zoe.clone()]);
        assert!(!list[0].protected);
        assert!(list[1].protected);

        assert_eq!(v.unlock(&joe.id, None).unwrap().entries().unwrap().len(), 1);
        assert!(matches!(
            v.unlock(&zoe.id, None),
            Err(StoreError::WrongPassword)
        ));
        let zoe_ledger = v.unlock(&zoe.id, Some("1234")).unwrap();
        assert_eq!(zoe_ledger.currency(), Currency::EUR);
        assert!(matches!(
            v.unlock("nobody", None),
            Err(StoreError::AccountNotFound)
        ));
        assert!(matches!(
            v.unlock("../escape", None),
            Err(StoreError::AccountNotFound)
        ));
    }

    #[test]
    fn categories_are_never_shared_between_accounts() {
        let (_d, v) = vault();
        let (_, mut joe) = v.create_account("Joe", None, Currency::USD).unwrap();
        joe.add_category("Rent", dindim_core::CategoryKind::Expense)
            .unwrap();
        let (_, ann) = v.create_account("Ann", None, Currency::USD).unwrap();
        assert!(ann.categories().unwrap().is_empty());
    }

    #[test]
    fn names_must_be_unique_and_valid() {
        let (_d, v) = vault();
        v.create_account("Joe", None, Currency::USD).unwrap();
        assert!(matches!(
            v.create_account("  joe ", None, Currency::USD),
            Err(StoreError::NameTaken)
        ));
        assert!(matches!(
            v.create_account("", None, Currency::USD),
            Err(StoreError::NameEmpty)
        ));
        let (other, _) = v.create_account("???", None, Currency::USD).unwrap();
        assert_eq!(other.id, "account");
    }

    #[test]
    fn renames_and_removes_accounts_to_trash() {
        let (_d, v) = vault();
        let (joe, _) = v.create_account("Joe", None, Currency::USD).unwrap();
        let (ann, _) = v.create_account("Ann", None, Currency::USD).unwrap();
        assert_eq!(v.rename_account(&joe.id, " Joseph ").unwrap(), "Joseph");
        assert!(matches!(
            v.rename_account(&joe.id, "ann"),
            Err(StoreError::NameTaken)
        ));
        assert_eq!(v.accounts().unwrap()[0].name, "Joseph");

        let trashed = v.remove_account(&ann.id).unwrap();
        assert!(trashed.exists());
        assert_eq!(v.accounts().unwrap().len(), 1);
        assert!(matches!(
            v.remove_account(&ann.id),
            Err(StoreError::AccountNotFound)
        ));
    }

    #[test]
    fn picks_up_files_dropped_into_the_folder() {
        let (_d, v) = vault();
        Ledger::create(&v.accounts_dir().join("spare.dindim"), None, Currency::USD).unwrap();
        fs::write(v.accounts_dir().join("readme.txt"), "hi").unwrap();
        let list = v.accounts().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "spare");
    }

    #[test]
    fn a_damaged_account_list_is_set_aside_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("data");
        {
            let (v, _) = Vault::open(&root).unwrap();
            v.create_account("Joe", None, Currency::USD).unwrap();
        }
        fs::write(root.join(MANIFEST), "{ not json").unwrap();

        let (v, damaged) = Vault::open(&root).unwrap();
        let damaged = damaged.expect("damaged list reported");
        assert_eq!(fs::read_to_string(&damaged).unwrap(), "{ not json");
        let list = v.accounts().unwrap();
        assert_eq!(list.len(), 1, "account file is still found");
        assert_eq!(list[0].id, "joe");
    }
}
