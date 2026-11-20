use thiserror::Error;

pub type Result<T, E = StoreError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("wrong password")]
    WrongPassword,
    #[error("account not found")]
    AccountNotFound,
    #[error("entry not found")]
    EntryNotFound,
    #[error("category not found")]
    CategoryNotFound,
    #[error("name is already in use")]
    NameTaken,
    #[error("name is empty")]
    NameEmpty,
    #[error("name is longer than {max} characters")]
    NameTooLong { max: usize },
    #[error("amount must be greater than zero")]
    AmountNotPositive,
    #[error("category does not match the entry type")]
    CategoryKindMismatch,
    #[error("file was created by a newer version of dindim")]
    TooNew,
    #[error("invalid backup: {0}")]
    InvalidBackup(BackupProblem),
    #[error("invalid CSV on line {line}: {problem}")]
    InvalidCsv { line: u64, problem: CsvProblem },
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BackupProblem {
    #[error("not a dindim backup")]
    NotABackup,
    #[error("archive is damaged")]
    Damaged,
    #[error("an account file is missing")]
    MissingAccount,
    #[error("an account file is not a dindim database")]
    NotADatabase,
    #[error("an account file is too large")]
    TooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CsvProblem {
    #[error("unexpected header")]
    Header,
    #[error("wrong number of columns")]
    Columns,
    #[error("invalid date")]
    Date,
    #[error("invalid type")]
    Type,
    #[error("invalid amount")]
    Amount,
    #[error("category name is invalid")]
    Category,
}

impl From<zip::result::ZipError> for StoreError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::Io(io) => StoreError::Io(io),
            zip::result::ZipError::FileNotFound => {
                StoreError::InvalidBackup(BackupProblem::MissingAccount)
            }
            _ => StoreError::InvalidBackup(BackupProblem::Damaged),
        }
    }
}
