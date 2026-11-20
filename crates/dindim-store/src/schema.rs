use rusqlite::Connection;

use crate::{Result, StoreError};

/// Each item upgrades the schema by one version. Released migrations are
/// never edited; changes go in a new item.
const MIGRATIONS: &[&str] = &[r#"
    CREATE TABLE category (
        id       INTEGER PRIMARY KEY,
        name     TEXT    NOT NULL,
        kind     TEXT    NOT NULL CHECK (kind IN ('expense', 'income')),
        position INTEGER NOT NULL DEFAULT 0,
        UNIQUE (name COLLATE NOCASE, kind)
    );

    CREATE TABLE entry (
        id          INTEGER PRIMARY KEY,
        date        TEXT    NOT NULL,
        kind        TEXT    NOT NULL
                    CHECK (kind IN ('expense', 'income', 'reserve_deposit', 'reserve_withdrawal')),
        amount      INTEGER NOT NULL CHECK (amount > 0),
        category_id INTEGER REFERENCES category (id) ON DELETE SET NULL,
        note        TEXT    NOT NULL DEFAULT '',
        created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    );

    CREATE INDEX entry_by_date ON entry (date);
    CREATE INDEX entry_by_category ON entry (category_id);

    CREATE TABLE setting (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
"#];

pub const CURRENT_VERSION: i64 = MIGRATIONS.len() as i64;

pub fn user_version(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("PRAGMA main.user_version", [], |r| r.get(0))?)
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    let version = user_version(conn)?;
    if version > CURRENT_VERSION {
        return Err(StoreError::TooNew);
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index as i64 + 1)?;
        tx.commit()?;
        tracing::info!(version = index + 1, "database schema migrated");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_fresh_database_and_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(user_version(&conn).unwrap(), CURRENT_VERSION);
        migrate(&mut conn).unwrap();
        assert_eq!(user_version(&conn).unwrap(), CURRENT_VERSION);
    }

    #[test]
    fn refuses_newer_databases() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", CURRENT_VERSION + 1)
            .unwrap();
        assert!(matches!(migrate(&mut conn), Err(StoreError::TooNew)));
    }
}
