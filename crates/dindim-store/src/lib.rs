//! Local storage for dindim.
//!
//! Every account is one SQLite file. Accounts with a password are encrypted
//! with SQLCipher, so the password protects the data on disk and not just the
//! screen.

pub mod backup;
pub mod csv;
mod error;
mod ledger;
mod schema;
mod vault;

pub use error::{BackupProblem, CsvProblem, Result, StoreError};
pub use ledger::{ImportSummary, Ledger, MAX_NAME_LEN, clean_name, is_encrypted};
pub use vault::{AccountInfo, Vault};
