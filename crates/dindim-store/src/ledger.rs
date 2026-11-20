use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use dindim_core::text::{fold, squish};
use dindim_core::{
    Category, CategoryId, CategoryKind, Currency, Date, Entry, EntryId, Kind, Money, NewEntry,
    OpeningBalance,
};
use rusqlite::{Connection, ErrorCode, OptionalExtension, Row, params};

use crate::schema::{self, CURRENT_VERSION};
use crate::{Result, StoreError};

pub(crate) const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
pub const MAX_NAME_LEN: usize = 60;

const KEY_CURRENCY: &str = "currency";
const KEY_OPENING_DATE: &str = "opening_balance_date";
const KEY_OPENING_AMOUNT: &str = "opening_balance_amount";

/// Whether the file at `path` is encrypted with a password.
pub fn is_encrypted(path: &Path) -> Result<bool> {
    let mut header = [0u8; 16];
    let read = fs::File::open(path)?.read(&mut header)?;
    Ok(read > 0 && &header != SQLITE_HEADER)
}

pub(crate) fn open_connection(path: &Path, password: Option<&str>) -> Result<Connection> {
    let conn = Connection::open(path)?;
    if let Some(password) = password.filter(|p| !p.is_empty()) {
        conn.pragma_update(None, "key", password)?;
    }
    match conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
        r.get::<_, i64>(0)
    }) {
        Ok(_) => {}
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == ErrorCode::NotADatabase => {
            return Err(StoreError::WrongPassword);
        }
        Err(e) => return Err(e.into()),
    }
    conn.pragma_update(None, "foreign_keys", true)?;
    Ok(conn)
}

/// Trims, collapses inner whitespace and checks the length of a name.
pub fn clean_name(raw: &str) -> Result<String> {
    let name = squish(raw);
    if name.is_empty() {
        return Err(StoreError::NameEmpty);
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(StoreError::NameTooLong { max: MAX_NAME_LEN });
    }
    Ok(name)
}

/// Totals of a CSV import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSummary {
    pub entries: usize,
    pub new_categories: usize,
}

/// One open account file.
pub struct Ledger {
    conn: Connection,
    path: PathBuf,
    currency: Currency,
}

impl std::fmt::Debug for Ledger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ledger")
            .field("path", &self.path)
            .field("currency", &self.currency)
            .finish_non_exhaustive()
    }
}

impl Ledger {
    pub fn create(path: &Path, password: Option<&str>, currency: Currency) -> Result<Self> {
        if path.exists() {
            return Err(StoreError::NameTaken);
        }
        let mut ledger = Self::connect(path, password)?;
        ledger.write_setting(KEY_CURRENCY, currency.code)?;
        ledger.currency = currency;
        Ok(ledger)
    }

    pub fn open(path: &Path, password: Option<&str>) -> Result<Self> {
        if !path.exists() {
            return Err(StoreError::AccountNotFound);
        }
        Self::connect(path, password)
    }

    pub fn in_memory(currency: Currency) -> Result<Self> {
        let mut ledger = Self::connect(Path::new(":memory:"), None)?;
        ledger.write_setting(KEY_CURRENCY, currency.code)?;
        ledger.currency = currency;
        Ok(ledger)
    }

    fn connect(path: &Path, password: Option<&str>) -> Result<Self> {
        let mut conn = open_connection(path, password)?;
        schema::migrate(&mut conn)?;
        let mut ledger = Ledger {
            conn,
            path: path.to_path_buf(),
            currency: Currency::USD,
        };
        if let Some(code) = ledger.setting(KEY_CURRENCY)? {
            ledger.currency = Currency::from_code(&code).unwrap_or(Currency::USD);
        }
        Ok(ledger)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    // Categories

    pub fn categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, kind FROM category ORDER BY kind, position, id")?;
        let rows = stmt.query_map([], |row| {
            let code: String = row.get(2)?;
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: CategoryKind::from_code(&code).unwrap_or(CategoryKind::Expense),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn category(&self, id: CategoryId) -> Result<Category> {
        self.categories()?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or(StoreError::CategoryNotFound)
    }

    fn find_category(&self, name: &str, kind: CategoryKind) -> Result<Option<Category>> {
        let wanted = fold(name);
        Ok(self
            .categories()?
            .into_iter()
            .find(|c| c.kind == kind && fold(&c.name) == wanted))
    }

    pub fn add_category(&mut self, name: &str, kind: CategoryKind) -> Result<Category> {
        let name = clean_name(name)?;
        if self.find_category(&name, kind)?.is_some() {
            return Err(StoreError::NameTaken);
        }
        let position: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM category WHERE kind = ?1",
            [kind.code()],
            |r| r.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO category (name, kind, position) VALUES (?1, ?2, ?3)",
            params![name, kind.code(), position],
        )?;
        Ok(Category {
            id: self.conn.last_insert_rowid(),
            name,
            kind,
        })
    }

    pub fn rename_category(&mut self, id: CategoryId, name: &str) -> Result<Category> {
        let name = clean_name(name)?;
        let mut category = self.category(id)?;
        if let Some(existing) = self.find_category(&name, category.kind)?
            && existing.id != id
        {
            return Err(StoreError::NameTaken);
        }
        self.conn.execute(
            "UPDATE category SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        category.name = name;
        Ok(category)
    }

    /// Deletes the category and returns the ids of the entries that lost it.
    pub fn delete_category(&mut self, id: CategoryId) -> Result<Vec<EntryId>> {
        let tx = self.conn.transaction()?;
        let affected: Vec<EntryId> = {
            let mut stmt = tx.prepare("SELECT id FROM entry WHERE category_id = ?1")?;
            stmt.query_map([id], |r| r.get(0))?
                .collect::<Result<_, _>>()?
        };
        tx.execute(
            "UPDATE entry SET category_id = NULL WHERE category_id = ?1",
            [id],
        )?;
        if tx.execute("DELETE FROM category WHERE id = ?1", [id])? == 0 {
            return Err(StoreError::CategoryNotFound);
        }
        tx.commit()?;
        Ok(affected)
    }

    /// Puts back a deleted category with its original id and entries.
    pub fn restore_category(&mut self, category: &Category, entries: &[EntryId]) -> Result<()> {
        let tx = self.conn.transaction()?;
        let position: i64 = tx.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM category WHERE kind = ?1",
            [category.kind.code()],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO category (id, name, kind, position) VALUES (?1, ?2, ?3, ?4)",
            params![category.id, category.name, category.kind.code(), position],
        )?;
        for entry in entries {
            tx.execute(
                "UPDATE entry SET category_id = ?1 WHERE id = ?2",
                params![category.id, entry],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // Entries

    /// Every entry, newest first.
    pub fn entries(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, kind, amount, category_id, note
             FROM entry ORDER BY date DESC, id DESC",
        )?;
        let rows = stmt.query_map([], row_to_entry)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn add_entry(&mut self, entry: &NewEntry) -> Result<Entry> {
        let entry = self.validate(entry.clone())?;
        self.conn.execute(
            "INSERT INTO entry (date, kind, amount, category_id, note)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.date.to_string(),
                entry.kind.code(),
                entry.amount.minor(),
                entry.category_id,
                entry.note,
            ],
        )?;
        Ok(entry.with_id(self.conn.last_insert_rowid()))
    }

    /// Puts back a deleted entry with its original id.
    pub fn restore_entry(&mut self, entry: &Entry) -> Result<()> {
        let valid = self.validate(entry.clone().into())?;
        self.conn.execute(
            "INSERT INTO entry (id, date, kind, amount, category_id, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id,
                valid.date.to_string(),
                valid.kind.code(),
                valid.amount.minor(),
                valid.category_id,
                valid.note,
            ],
        )?;
        Ok(())
    }

    pub fn update_entry(&mut self, entry: &Entry) -> Result<Entry> {
        let valid = self.validate(entry.clone().into())?;
        let changed = self.conn.execute(
            "UPDATE entry SET date = ?1, kind = ?2, amount = ?3, category_id = ?4, note = ?5
             WHERE id = ?6",
            params![
                valid.date.to_string(),
                valid.kind.code(),
                valid.amount.minor(),
                valid.category_id,
                valid.note,
                entry.id,
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::EntryNotFound);
        }
        Ok(valid.with_id(entry.id))
    }

    pub fn delete_entry(&mut self, id: EntryId) -> Result<()> {
        if self.conn.execute("DELETE FROM entry WHERE id = ?1", [id])? == 0 {
            return Err(StoreError::EntryNotFound);
        }
        Ok(())
    }

    fn validate(&self, mut entry: NewEntry) -> Result<NewEntry> {
        if !entry.amount.is_positive() {
            return Err(StoreError::AmountNotPositive);
        }
        entry.note = squish(&entry.note);
        match (entry.kind.category_kind(), entry.category_id) {
            (None, _) => entry.category_id = None,
            (Some(expected), Some(id)) => {
                if self.category(id)?.kind != expected {
                    return Err(StoreError::CategoryKindMismatch);
                }
            }
            (Some(_), None) => {}
        }
        Ok(entry)
    }

    // Opening balance

    pub fn opening_balance(&self) -> Result<Option<OpeningBalance>> {
        let date = self.setting(KEY_OPENING_DATE)?;
        let amount = self.setting(KEY_OPENING_AMOUNT)?;
        Ok(date.zip(amount).and_then(|(date, amount)| {
            Some(OpeningBalance {
                date: date.parse::<Date>().ok()?,
                amount: Money::from_minor(amount.parse().ok()?),
            })
        }))
    }

    pub fn set_opening_balance(&mut self, opening: Option<OpeningBalance>) -> Result<()> {
        let tx = self.conn.transaction()?;
        match opening {
            Some(o) => {
                for (key, value) in [
                    (KEY_OPENING_DATE, o.date.to_string()),
                    (KEY_OPENING_AMOUNT, o.amount.minor().to_string()),
                ] {
                    tx.execute(
                        "INSERT INTO setting (key, value) VALUES (?1, ?2)
                         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                        params![key, value],
                    )?;
                }
            }
            None => {
                tx.execute(
                    "DELETE FROM setting WHERE key IN (?1, ?2)",
                    params![KEY_OPENING_DATE, KEY_OPENING_AMOUNT],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    fn write_setting(&mut self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // CSV

    /// Adds every row in one transaction, creating missing categories.
    pub fn import_rows(&mut self, rows: &[crate::csv::CsvRow]) -> Result<ImportSummary> {
        let mut known = self.categories()?;
        let mut summary = ImportSummary::default();
        let tx = self.conn.transaction()?;
        for row in rows {
            let category_id = match (row.kind.category_kind(), &row.category) {
                (Some(kind), Some(name)) => {
                    let wanted = fold(name);
                    match known
                        .iter()
                        .find(|c| c.kind == kind && fold(&c.name) == wanted)
                    {
                        Some(c) => Some(c.id),
                        None => {
                            let name = clean_name(name)?;
                            tx.execute(
                                "INSERT INTO category (name, kind, position)
                                 VALUES (?1, ?2, (SELECT COALESCE(MAX(position), -1) + 1
                                                  FROM category WHERE kind = ?2))",
                                params![name, kind.code()],
                            )?;
                            let id = tx.last_insert_rowid();
                            known.push(Category { id, name, kind });
                            summary.new_categories += 1;
                            Some(id)
                        }
                    }
                }
                _ => None,
            };
            tx.execute(
                "INSERT INTO entry (date, kind, amount, category_id, note)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    row.date.to_string(),
                    row.kind.code(),
                    row.amount.minor(),
                    category_id,
                    squish(&row.note),
                ],
            )?;
            summary.entries += 1;
        }
        tx.commit()?;
        Ok(summary)
    }

    // Password

    /// Sets, changes or removes (`None`) the password of this file.
    ///
    /// The data is copied into a new file with the new key, which replaces the
    /// old one only after it opens correctly. If the final swap fails the
    /// ledger is left closed and the original file stays untouched.
    pub fn set_password(&mut self, new_password: Option<&str>) -> Result<()> {
        let new_password = new_password.filter(|p| !p.is_empty());
        let tmp = self.path.with_extension("rekey");
        if tmp.exists() {
            fs::remove_file(&tmp)?;
        }
        let exported = self.export_to(&tmp, new_password.unwrap_or(""));
        if let Err(e) = exported.and_then(|()| open_connection(&tmp, new_password).map(drop)) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        fs::File::open(&tmp)?.sync_all()?;

        drop(std::mem::replace(
            &mut self.conn,
            Connection::open_in_memory()?,
        ));
        fs::rename(&tmp, &self.path)?;
        sync_parent_dir(&self.path);
        self.conn = open_connection(&self.path, new_password)?;
        Ok(())
    }

    fn export_to(&self, target: &Path, key: &str) -> Result<()> {
        self.conn.execute(
            "ATTACH DATABASE ?1 AS rekey KEY ?2",
            params![target.to_string_lossy(), key],
        )?;
        let exported = self
            .conn
            .query_row("SELECT sqlcipher_export('rekey')", [], |_| Ok(()))
            .and_then(|()| {
                self.conn
                    .pragma_update(Some("rekey"), "user_version", CURRENT_VERSION)
            });
        self.conn.execute("DETACH DATABASE rekey", [])?;
        Ok(exported?)
    }
}

pub(crate) fn sync_parent_dir(path: &Path) {
    #[cfg(unix)]
    if let Some(parent) = path.parent()
        && let Ok(dir) = fs::File::open(parent)
    {
        let _ = dir.sync_all();
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<Entry> {
    let bad = |col: usize, msg: String| {
        rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, msg.into())
    };
    let date: String = row.get(1)?;
    let kind: String = row.get(2)?;
    Ok(Entry {
        id: row.get(0)?,
        date: date
            .parse()
            .map_err(|e| bad(1, format!("invalid date {date:?}: {e}")))?,
        kind: Kind::from_code(&kind).ok_or_else(|| bad(2, format!("invalid kind {kind:?}")))?,
        amount: Money::from_minor(row.get(3)?),
        category_id: row.get(4)?,
        note: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn ledger() -> Ledger {
        Ledger::in_memory(Currency::USD).unwrap()
    }

    fn new_entry(kind: Kind, minor: i64, category_id: Option<i64>) -> NewEntry {
        NewEntry {
            date: date(2026, 11, 19),
            kind,
            amount: Money::from_minor(minor),
            category_id,
            note: "  milk   and bread ".into(),
        }
    }

    #[test]
    fn new_accounts_start_without_categories() {
        assert!(ledger().categories().unwrap().is_empty());
    }

    #[test]
    fn remembers_its_currency() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.dindim");
        let brl = Currency::from_code("BRL").unwrap();
        drop(Ledger::create(&path, None, brl).unwrap());
        assert_eq!(Ledger::open(&path, None).unwrap().currency(), brl);
    }

    #[test]
    fn adds_lists_updates_and_deletes_entries() {
        let mut l = ledger();
        let groceries = l.add_category("Groceries", CategoryKind::Expense).unwrap();
        let saved = l
            .add_entry(&new_entry(Kind::Expense, 1250, Some(groceries.id)))
            .unwrap();
        assert_eq!(saved.note, "milk and bread");
        assert_eq!(l.entries().unwrap(), vec![saved.clone()]);

        let mut changed = saved.clone();
        changed.amount = Money::from_minor(990);
        changed.date = date(2026, 11, 1);
        assert_eq!(l.update_entry(&changed).unwrap(), changed);
        assert_eq!(l.entries().unwrap()[0], changed);

        l.delete_entry(saved.id).unwrap();
        assert!(l.entries().unwrap().is_empty());
        assert!(matches!(
            l.delete_entry(saved.id),
            Err(StoreError::EntryNotFound)
        ));
        assert!(matches!(
            l.update_entry(&changed),
            Err(StoreError::EntryNotFound)
        ));

        l.restore_entry(&changed).unwrap();
        assert_eq!(l.entries().unwrap(), vec![changed]);
    }

    #[test]
    fn entries_come_newest_first() {
        let mut l = ledger();
        for day in [3, 1, 2] {
            let mut e = new_entry(Kind::Expense, 100, None);
            e.date = date(2026, 11, day);
            l.add_entry(&e).unwrap();
        }
        let days: Vec<_> = l.entries().unwrap().iter().map(|e| e.date.day()).collect();
        assert_eq!(days, vec![3, 2, 1]);
    }

    #[test]
    fn validates_entries() {
        let mut l = ledger();
        let salary = l.add_category("Salary", CategoryKind::Income).unwrap();
        assert!(matches!(
            l.add_entry(&new_entry(Kind::Expense, 0, None)),
            Err(StoreError::AmountNotPositive)
        ));
        assert!(matches!(
            l.add_entry(&new_entry(Kind::Expense, 100, Some(salary.id))),
            Err(StoreError::CategoryKindMismatch)
        ));
        assert!(matches!(
            l.add_entry(&new_entry(Kind::Expense, 100, Some(9999))),
            Err(StoreError::CategoryNotFound)
        ));
        let saved = l
            .add_entry(&new_entry(Kind::ReserveDeposit, 100, Some(salary.id)))
            .unwrap();
        assert_eq!(saved.category_id, None);
    }

    #[test]
    fn manages_categories() {
        let mut l = ledger();
        let pets = l.add_category("  Pets  ", CategoryKind::Expense).unwrap();
        assert_eq!(pets.name, "Pets");
        assert!(matches!(
            l.add_category("pets", CategoryKind::Expense),
            Err(StoreError::NameTaken)
        ));
        l.add_category("Café", CategoryKind::Expense).unwrap();
        assert!(matches!(
            l.add_category("cafe", CategoryKind::Expense),
            Err(StoreError::NameTaken)
        ));
        l.add_category("Pets", CategoryKind::Income).unwrap();
        assert!(matches!(
            l.add_category("   ", CategoryKind::Expense),
            Err(StoreError::NameEmpty)
        ));
        assert!(matches!(
            l.add_category(&"x".repeat(MAX_NAME_LEN + 1), CategoryKind::Expense),
            Err(StoreError::NameTooLong { .. })
        ));

        assert_eq!(
            l.rename_category(pets.id, "Animals").unwrap().name,
            "Animals"
        );
        l.rename_category(pets.id, "animals").unwrap();
        assert!(matches!(
            l.rename_category(pets.id, "Cafe"),
            Err(StoreError::NameTaken)
        ));
        assert!(matches!(
            l.rename_category(4242, "X"),
            Err(StoreError::CategoryNotFound)
        ));
    }

    #[test]
    fn deleting_a_category_keeps_entries_and_can_be_undone() {
        let mut l = ledger();
        let pets = l.add_category("Pets", CategoryKind::Expense).unwrap();
        let entry = l
            .add_entry(&new_entry(Kind::Expense, 100, Some(pets.id)))
            .unwrap();
        assert_eq!(l.delete_category(pets.id).unwrap(), vec![entry.id]);
        assert_eq!(l.entries().unwrap()[0].category_id, None);
        assert!(matches!(
            l.delete_category(pets.id),
            Err(StoreError::CategoryNotFound)
        ));

        l.restore_category(&pets, &[entry.id]).unwrap();
        assert!(l.categories().unwrap().contains(&pets));
        assert_eq!(l.entries().unwrap()[0].category_id, Some(pets.id));
    }

    #[test]
    fn stores_opening_balance() {
        let mut l = ledger();
        assert_eq!(l.opening_balance().unwrap(), None);
        let o = OpeningBalance {
            date: date(2026, 4, 1),
            amount: Money::from_minor(165_038),
        };
        l.set_opening_balance(Some(o)).unwrap();
        assert_eq!(l.opening_balance().unwrap(), Some(o));
        l.set_opening_balance(None).unwrap();
        assert_eq!(l.opening_balance().unwrap(), None);
    }

    #[test]
    fn password_can_be_added_changed_and_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("account.dindim");
        let mut l = Ledger::create(&path, None, Currency::USD).unwrap();
        l.add_category("Rent", CategoryKind::Expense).unwrap();
        l.add_entry(&new_entry(Kind::Expense, 100, None)).unwrap();
        assert!(!is_encrypted(&path).unwrap());

        l.set_password(Some("secret")).unwrap();
        assert!(is_encrypted(&path).unwrap());
        assert_eq!(l.entries().unwrap().len(), 1);
        drop(l);

        assert!(matches!(
            Ledger::open(&path, None),
            Err(StoreError::WrongPassword)
        ));
        assert!(matches!(
            Ledger::open(&path, Some("wrong")),
            Err(StoreError::WrongPassword)
        ));
        let mut l = Ledger::open(&path, Some("secret")).unwrap();
        assert_eq!(l.categories().unwrap().len(), 1);
        assert_eq!(l.currency(), Currency::USD);

        l.set_password(Some("another")).unwrap();
        drop(l);
        assert!(Ledger::open(&path, Some("secret")).is_err());
        let mut l = Ledger::open(&path, Some("another")).unwrap();

        l.set_password(None).unwrap();
        drop(l);
        assert!(!is_encrypted(&path).unwrap());
        let l = Ledger::open(&path, None).unwrap();
        assert_eq!(l.entries().unwrap().len(), 1);
        assert_eq!(schema::user_version(&l.conn).unwrap(), CURRENT_VERSION);
        assert!(!path.with_extension("rekey").exists());
    }

    #[test]
    fn create_refuses_to_overwrite_and_open_requires_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.dindim");
        Ledger::create(&path, None, Currency::USD).unwrap();
        assert!(matches!(
            Ledger::create(&path, None, Currency::USD),
            Err(StoreError::NameTaken)
        ));
        assert!(matches!(
            Ledger::open(&dir.path().join("missing.dindim"), None),
            Err(StoreError::AccountNotFound)
        ));
    }
}
