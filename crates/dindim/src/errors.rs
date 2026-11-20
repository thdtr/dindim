//! Turns errors into messages for people. Technical details go to the log.

use dindim_core::AmountError;
use dindim_i18n::{I18n, args};
use dindim_store::{BackupProblem, CsvProblem, StoreError};

pub fn amount(i18n: &I18n, error: AmountError) -> String {
    i18n.t(match error {
        AmountError::Empty => "error-amount-empty",
        AmountError::Invalid => "error-amount-invalid",
        AmountError::TooManyDecimals => "error-amount-decimals",
        AmountError::Overflow => "error-amount-too-large",
    })
}

pub fn store(i18n: &I18n, error: &StoreError) -> String {
    match error {
        StoreError::Database(_) | StoreError::Io(_) => {
            tracing::error!(error = %error, "storage error");
        }
        _ => tracing::debug!(error = %error, "rejected by storage"),
    }
    match error {
        StoreError::WrongPassword => i18n.t("error-wrong-password"),
        StoreError::AccountNotFound => i18n.t("error-account-not-found"),
        StoreError::EntryNotFound => i18n.t("error-entry-not-found"),
        StoreError::CategoryNotFound => i18n.t("error-category-not-found"),
        StoreError::NameTaken => i18n.t("error-name-taken"),
        StoreError::NameEmpty => i18n.t("error-name-empty"),
        StoreError::NameTooLong { max } => {
            i18n.t_with("error-name-too-long", &args! { "max" => *max })
        }
        StoreError::AmountNotPositive => i18n.t("error-amount-not-positive"),
        StoreError::CategoryKindMismatch => i18n.t("error-category-kind"),
        StoreError::TooNew => i18n.t("error-too-new"),
        StoreError::InvalidBackup(problem) => i18n.t(match problem {
            BackupProblem::NotABackup => "error-backup-not-backup",
            BackupProblem::Damaged => "error-backup-damaged",
            BackupProblem::MissingAccount => "error-backup-missing-account",
            BackupProblem::NotADatabase => "error-backup-not-database",
            BackupProblem::TooLarge => "error-backup-too-large",
        }),
        StoreError::InvalidCsv { line, problem } => {
            let reason = i18n.t(match problem {
                CsvProblem::Header => "error-csv-header",
                CsvProblem::Columns => "error-csv-columns",
                CsvProblem::Date => "error-csv-date",
                CsvProblem::Type => "error-csv-type",
                CsvProblem::Amount => "error-csv-amount",
                CsvProblem::Category => "error-csv-category",
            });
            i18n.t_with(
                "error-csv-line",
                &args! { "line" => line.to_string(), "reason" => reason },
            )
        }
        StoreError::Database(_) => i18n.t("error-database"),
        StoreError::Io(_) => i18n.t("error-io"),
    }
}
