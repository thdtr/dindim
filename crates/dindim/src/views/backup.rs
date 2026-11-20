use std::sync::LazyLock;

use iced::widget::svg::Handle;
use iced::widget::{Button, button, column, container, row, space, text};
use iced::{Alignment, Element, Length};

use super::{card, centered, divider, labeled, notice};
use crate::app::{App, Message};
use crate::icons::{self, icon};
use crate::theme::{self, Ink, size};

fn action_row<'a>(
    glyph: &LazyLock<Handle>,
    title: String,
    description: String,
    action: Button<'a, Message>,
) -> Element<'a, Message> {
    row![
        container(icon(glyph, 20.0, Ink::Primary))
            .center(44)
            .style(theme::fill_sunk(12.0)),
        column![
            text(title).size(size::LEAD).font(theme::SANS_SEMI),
            super::muted(description),
        ]
        .spacing(3)
        .width(Length::Fill),
        action,
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let folder = app
        .vault
        .as_ref()
        .map(|v| v.root().display().to_string())
        .unwrap_or_default();
    let mut page = column![
        text(i18n.t("backup-title"))
            .font(theme::DISPLAY_BOLD)
            .size(size::HERO),
        text(i18n.t("backup-intro"))
            .size(size::LEAD)
            .style(theme::ink(Ink::Secondary)),
    ]
    .spacing(10);
    if let Some(n) = notice(&app.notice) {
        page = page.push(n);
    }

    page = page.push(space().height(6)).push(card(
        column![
            action_row(
                &icons::DOWNLOAD,
                i18n.t("backup-export-title"),
                i18n.t("backup-export-description"),
                button(labeled(
                    &icons::DOWNLOAD,
                    i18n.t("backup-export"),
                    Ink::OnGold
                ))
                .padding([12, 18])
                .style(theme::primary)
                .on_press(Message::ExportBackup),
            ),
            divider(),
            action_row(
                &icons::UPLOAD,
                i18n.t("backup-import-title"),
                i18n.t("backup-import-description"),
                button(labeled(
                    &icons::UPLOAD,
                    i18n.t("backup-import"),
                    Ink::Primary
                ))
                .padding([12, 18])
                .style(theme::secondary)
                .on_press_maybe((!app.busy).then_some(Message::ImportBackup)),
            ),
        ]
        .spacing(20),
    ));

    let auto = if app.prefs.auto_backup {
        i18n.t_with(
            "backup-auto-on",
            &dindim_i18n::args! { "count" => app.prefs.backups_to_keep },
        )
    } else {
        i18n.t("backup-auto-off")
    };
    page = page.push(
        container(
            column![
                row![
                    super::muted(i18n.t("backup-folder")),
                    text(folder).size(size::SMALL).font(theme::NUM),
                    space::horizontal(),
                    button(text(i18n.t("backup-open-folder")).size(size::SMALL))
                        .padding(0)
                        .style(theme::link)
                        .on_press(Message::OpenDataFolder),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
                super::muted(auto),
            ]
            .spacing(6),
        )
        .padding([12, 14])
        .style(theme::sunk),
    );
    centered(page, 720.0)
}
