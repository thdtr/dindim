use iced::widget::{column, container, pick_list, row, text, toggler};
use iced::{Alignment, Element, Length};

use super::{Choice, card, centered, divider, setting_row};
use crate::app::{App, Message};
use crate::preferences::{BACKUP_KEEP_CHOICES, LOCK_CHOICES, Language, ThemeChoice};
use crate::theme::{self, Ink, size};

fn picker<'a, T: Clone + PartialEq + 'static>(
    choices: Vec<Choice<T>>,
    current: &T,
    on_pick: fn(T) -> Message,
) -> Element<'a, Message> {
    let selected = choices.iter().find(|c| &c.value == current).cloned();
    pick_list(choices, selected, move |c: Choice<T>| on_pick(c.value))
        .padding([10, 14])
        .text_size(size::BODY)
        .width(Length::Fill)
        .style(theme::pick)
        .menu_style(theme::pick_menu)
        .into()
}

fn switch<'a>(label: String, on: bool, on_toggle: fn(bool) -> Message) -> Element<'a, Message> {
    row![
        toggler(on).on_toggle(on_toggle).size(22),
        text(label).size(size::BODY),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let prefs = &app.prefs;

    let languages: Vec<Choice<Language>> = Language::ALL
        .iter()
        .map(|l| Choice {
            value: *l,
            label: match l.fixed() {
                Some(locale) => locale.native_name().to_string(),
                None => i18n.t("prefs-language-system"),
            },
        })
        .collect();
    let themes: Vec<Choice<ThemeChoice>> = ThemeChoice::ALL
        .iter()
        .map(|t| Choice {
            value: *t,
            label: i18n.t(match t {
                ThemeChoice::System => "prefs-theme-system",
                ThemeChoice::Light => "prefs-theme-light",
                ThemeChoice::Dark => "prefs-theme-dark",
            }),
        })
        .collect();
    let locks: Vec<Choice<u32>> = LOCK_CHOICES
        .iter()
        .map(|m| Choice {
            value: *m,
            label: if *m == 0 {
                i18n.t("prefs-lock-never")
            } else {
                i18n.t_with(
                    "prefs-lock-minutes",
                    &dindim_i18n::args! { "minutes" => *m },
                )
            },
        })
        .collect();
    let keeps: Vec<Choice<usize>> = BACKUP_KEEP_CHOICES
        .iter()
        .map(|n| Choice {
            value: *n,
            label: i18n.t_with("prefs-backup-keep", &dindim_i18n::args! { "count" => *n }),
        })
        .collect();

    let zoom = row![
        super::icon_button(&crate::icons::MINUS, Message::ZoomOut),
        container(
            text(format!("{:.0}%", prefs.zoom * 100.0))
                .size(size::BODY)
                .font(theme::NUM)
        )
        .center_x(80),
        super::icon_button(&crate::icons::PLUS, Message::ZoomIn),
        iced::widget::button(text(i18n.t("prefs-zoom-reset")).size(size::SMALL))
            .padding([8, 12])
            .style(theme::secondary)
            .on_press(Message::ZoomReset),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let backup = column![
        switch(
            i18n.t("prefs-backup-auto"),
            prefs.auto_backup,
            Message::SetAutoBackup
        ),
        picker(keeps, &prefs.backups_to_keep, Message::SetBackupsToKeep),
    ]
    .spacing(12);

    let shortcuts = [
        ("Ctrl+N", "shortcut-new-entry"),
        ("Ctrl+F", "shortcut-find"),
        ("Ctrl+Z", "shortcut-undo"),
        ("Ctrl+Shift+Z", "shortcut-redo"),
        ("Ctrl+1…4", "shortcut-tabs"),
        ("PageUp / PageDown", "shortcut-months"),
        ("Ctrl+L", "shortcut-lock"),
        ("Ctrl+,", "shortcut-preferences"),
        ("Ctrl+= / Ctrl+-", "shortcut-zoom"),
        ("Esc", "shortcut-escape"),
        ("Ctrl+Q", "shortcut-quit"),
    ];
    let mut keys = column![].spacing(8);
    for (combo, key) in shortcuts {
        let combo = if cfg!(target_os = "macos") {
            combo.replace("Ctrl", "⌘")
        } else {
            combo.to_string()
        };
        keys = keys.push(
            row![
                container(text(combo).size(size::SMALL).font(theme::NUM))
                    .padding([3, 8])
                    .width(170)
                    .style(theme::sunk),
                text(i18n.t(key)).size(size::SMALL),
            ]
            .spacing(14)
            .align_y(Alignment::Center),
        );
    }

    let page = column![
        text(i18n.t("prefs-title"))
            .font(theme::DISPLAY_BOLD)
            .size(size::HERO),
        card(
            column![
                setting_row(
                    i18n.t("prefs-language-title"),
                    i18n.t("prefs-language-description"),
                    picker(languages, &prefs.language, Message::SetLanguage),
                ),
                divider(),
                setting_row(
                    i18n.t("prefs-theme-title"),
                    i18n.t("prefs-theme-description"),
                    picker(themes, &prefs.theme, Message::SetTheme),
                ),
                divider(),
                setting_row(
                    i18n.t("prefs-zoom-title"),
                    i18n.t("prefs-zoom-description"),
                    zoom,
                ),
                divider(),
                setting_row(
                    i18n.t("prefs-lock-title"),
                    i18n.t("prefs-lock-description"),
                    picker(locks, &prefs.lock_after_minutes, Message::SetLockAfter),
                ),
                divider(),
                setting_row(
                    i18n.t("prefs-backup-title"),
                    i18n.t("prefs-backup-description"),
                    backup,
                ),
                divider(),
                setting_row(
                    i18n.t("prefs-updates-title"),
                    i18n.t("prefs-updates-description"),
                    switch(
                        i18n.t("prefs-updates-toggle"),
                        prefs.check_for_updates,
                        Message::SetCheckUpdates
                    ),
                ),
            ]
            .spacing(24)
        )
        .padding(28),
        card(
            column![
                text(i18n.t("prefs-shortcuts-title"))
                    .font(theme::DISPLAY)
                    .size(size::TITLE),
                keys,
            ]
            .spacing(16)
        )
        .padding(28),
        text(i18n.t("prefs-saved-note"))
            .size(size::SMALL)
            .style(theme::ink(Ink::Faint)),
    ]
    .spacing(18);
    centered(page, 920.0)
}
