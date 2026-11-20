use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use dindim_i18n::args;
use dindim_store::{StoreError, backup, csv};
use iced::Task;

use super::{AUTO_BACKUP_INTERVAL, App, Message};
use crate::handoff::Handoff;
use crate::session::Notice;

/// Runs blocking file work off the UI thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> dindim_store::Result<T> + Send + 'static,
) -> Handoff<dindim_store::Result<T>> {
    let result = tokio::task::spawn_blocking(work)
        .await
        .unwrap_or_else(|e| Err(StoreError::Io(std::io::Error::other(e))));
    Handoff::new(result)
}

impl App {
    pub(super) fn update_data(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ExportCsv => {
                let Some(acc) = self.active() else {
                    return Task::none();
                };
                let name = format!("{}-{}.csv", acc.info.id, self.today);
                let title = self.i18n.t("dialog-export-csv");
                let filter = self.i18n.t("dialog-csv-files");
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter(filter, &["csv"])
                            .set_file_name(name)
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ExportCsvTo,
                );
            }
            Message::ExportCsvTo(Some(path)) => {
                let path = with_extension(path, "csv");
                let Some(acc) = self.active() else {
                    return Task::none();
                };
                let mut bytes = Vec::new();
                if let Err(e) = csv::write(&mut bytes, &acc.entries, &acc.categories, acc.currency)
                {
                    let text = self.store_error(&e);
                    self.toast(text, false);
                    return Task::none();
                }
                let target = path.clone();
                return Task::perform(
                    blocking(move || Ok(std::fs::write(&target, bytes)?)),
                    move |result| Message::CsvWritten(path.clone(), result),
                );
            }
            Message::CsvWritten(path, result) => match result.take() {
                Some(Ok(())) => {
                    let text = self.i18n.t_with(
                        "toast-csv-exported",
                        &args! { "path" => path.display().to_string() },
                    );
                    self.toast(text, true);
                }
                Some(Err(e)) => {
                    let text = self.store_error(&e);
                    self.toast(text, false);
                }
                None => {}
            },
            Message::ImportCsv => {
                let title = self.i18n.t("dialog-import-csv");
                let filter = self.i18n.t("dialog-csv-files");
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter(filter, &["csv"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ImportCsvFrom,
                );
            }
            Message::ImportCsvFrom(Some(path)) => {
                let Some(acc) = self.active() else {
                    return Task::none();
                };
                let currency = acc.currency;
                let id = acc.info.id.clone();
                self.busy = true;
                return Task::perform(
                    blocking(move || csv::read(std::fs::File::open(&path)?, currency)),
                    move |result| Message::CsvRead(id.clone(), result),
                );
            }
            Message::CsvRead(id, result) => {
                self.busy = false;
                let Some(result) = result.take() else {
                    return Task::none();
                };
                let App { i18n, open, .. } = self;
                let Some(acc) = open.iter_mut().find(|a| a.info.id == id) else {
                    return Task::none();
                };
                let i18n = &*i18n;
                let outcome = result
                    .and_then(|rows| acc.ledger.import_rows(&rows))
                    .and_then(|summary| acc.reload().map(|()| summary));
                acc.settings.notice = Some(match outcome {
                    Ok(summary) => {
                        acc.history.clear();
                        tracing::info!(entries = summary.entries, "csv imported");
                        let mut text = i18n.t_with(
                            "notice-csv-imported",
                            &args! { "entries" => summary.entries },
                        );
                        if summary.new_categories > 0 {
                            text.push(' ');
                            text.push_str(&i18n.t_with(
                                "notice-csv-new-categories",
                                &args! { "count" => summary.new_categories },
                            ));
                        }
                        Notice::ok(text)
                    }
                    Err(e) => Notice::error(crate::errors::store(i18n, &e)),
                });
            }

            Message::ExportBackup => {
                let name = backup::file_name(self.now_civil());
                let title = self.i18n.t("dialog-export-backup");
                let filter = self.i18n.t("dialog-backup-files");
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter(filter, &["zip"])
                            .set_file_name(name)
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ExportBackupTo,
                );
            }
            Message::ExportBackupTo(Some(path)) => {
                return self.write_backup(with_extension(path, "zip"), false);
            }
            Message::BackupWritten {
                path,
                count,
                automatic,
                result,
            } => {
                let Some(result) = result.take() else {
                    return Task::none();
                };
                match (result, automatic) {
                    (Ok(()), true) => {
                        tracing::info!(path = %path.display(), accounts = count, "automatic backup saved");
                        if let Some(vault) = &self.vault
                            && let Err(e) =
                                backup::prune(&vault.backups_dir(), self.prefs.backups_to_keep)
                        {
                            tracing::warn!(error = %e, "could not prune old backups");
                        }
                    }
                    (Ok(()), false) => {
                        tracing::info!(accounts = count, "backup exported");
                        self.notice = Some(Notice::ok(self.i18n.t_with(
                            "notice-backup-saved",
                            &args! { "count" => count, "path" => path.display().to_string() },
                        )));
                    }
                    (Err(e), true) => {
                        tracing::warn!(error = %e, "automatic backup failed");
                    }
                    (Err(e), false) => self.notice = Some(Notice::error(self.store_error(&e))),
                }
            }
            Message::ImportBackup => {
                let title = self.i18n.t("dialog-import-backup");
                let filter = self.i18n.t("dialog-backup-files");
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter(filter, &["zip"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ImportBackupFrom,
                );
            }
            Message::ImportBackupFrom(Some(path)) => {
                self.busy = true;
                return Task::perform(blocking(move || backup::read(&path)), Message::BackupRead);
            }
            Message::BackupRead(result) => {
                self.busy = false;
                let Some(result) = result.take() else {
                    return Task::none();
                };
                let Some(vault) = self.vault.clone() else {
                    return Task::none();
                };
                self.notice = Some(match result.and_then(|contents| vault.install(contents)) {
                    Ok(list) if list.is_empty() => Notice::ok(self.i18n.t("notice-backup-empty")),
                    Ok(list) => {
                        let names: Vec<&str> = list.iter().map(|a| a.name.as_str()).collect();
                        Notice::ok(self.i18n.t_with(
                            "notice-backup-imported",
                            &args! { "count" => list.len(), "names" => names.join(", ") },
                        ))
                    }
                    Err(e) => Notice::error(self.store_error(&e)),
                });
                self.refresh_accounts();
            }
            Message::OpenDataFolder => {
                if let Some(vault) = &self.vault
                    && let Err(e) = open::that_detached(vault.root())
                {
                    tracing::warn!(error = %e, "could not open data folder");
                }
            }
            Message::ExportCsvTo(None)
            | Message::ImportCsvFrom(None)
            | Message::ExportBackupTo(None)
            | Message::ImportBackupFrom(None) => {}
            _ => {}
        }
        Task::none()
    }

    fn write_backup(&mut self, path: PathBuf, automatic: bool) -> Task<Message> {
        let Some(vault) = &self.vault else {
            return Task::none();
        };
        let snapshot = match vault.snapshot() {
            Ok(snapshot) => snapshot,
            Err(e) => {
                if automatic {
                    tracing::warn!(error = %e, "automatic backup failed");
                } else {
                    self.notice = Some(Notice::error(self.store_error(&e)));
                }
                return Task::none();
            }
        };
        if automatic && snapshot.is_empty() {
            return Task::none();
        }
        let count = snapshot.len();
        let target = path.clone();
        Task::perform(
            blocking(move || {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                snapshot.write_zip(&target)
            }),
            move |result| Message::BackupWritten {
                path: path.clone(),
                count,
                automatic,
                result,
            },
        )
    }

    /// Takes an automatic backup when the newest one is a day old or missing.
    pub(super) fn maybe_auto_backup(&mut self) -> Task<Message> {
        let due_check = self
            .last_backup_check
            .is_none_or(|t| t.elapsed() >= std::time::Duration::from_secs(60 * 60));
        if !self.prefs.auto_backup || !due_check || self.accounts.is_empty() {
            return Task::none();
        }
        self.last_backup_check = Some(Instant::now());
        let Some(dir) = self.vault.as_ref().map(|v| v.backups_dir()) else {
            return Task::none();
        };
        let newest_age = backup::list(&dir)
            .ok()
            .and_then(|list| list.into_iter().next())
            .and_then(|p| p.metadata().ok()?.modified().ok())
            .and_then(|t| SystemTime::now().duration_since(t).ok());
        if newest_age.is_some_and(|age| age < AUTO_BACKUP_INTERVAL) {
            return Task::none();
        }
        let path = dir.join(backup::file_name(self.now_civil()));
        self.write_backup(path, true)
    }

    fn now_civil(&self) -> jiff::civil::DateTime {
        jiff::ZonedDateTime::now().datetime()
    }
}

fn with_extension(path: PathBuf, extension: &str) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension(extension)
    } else {
        path
    }
}
