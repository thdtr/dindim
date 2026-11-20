use iced::widget::{button, column, row, space, text};
use iced::{Alignment, Element};

use super::{card, centered, labeled, wordmark};
use crate::app::{App, Message};
use crate::icons;
use crate::theme::{self, Ink, size};
use crate::updates::RELEASES_PAGE;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub fn view(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let link = |label: String, url: String| {
        button(labeled(&icons::EXTERNAL, label, Ink::GoldText))
            .padding([6, 0])
            .style(theme::link)
            .on_press(Message::OpenUrl(url))
    };
    let page = column![
        wordmark(56.0, 44.0, false),
        text(i18n.t_with(
            "about-version",
            &dindim_i18n::args! { "version" => env!("CARGO_PKG_VERSION") },
        ))
        .size(size::BODY)
        .font(theme::NUM)
        .style(theme::ink(Ink::Secondary)),
        text(i18n.t("about-description"))
            .size(size::LEAD)
            .style(theme::ink(Ink::Secondary)),
        space().height(4),
        card(
            column![
                link(i18n.t("about-source"), REPOSITORY.to_string()),
                link(i18n.t("about-releases"), RELEASES_PAGE.to_string()),
                link(
                    i18n.t("about-report-issue"),
                    format!("{REPOSITORY}/issues/new/choose")
                ),
                link(
                    i18n.t("about-license"),
                    format!("{REPOSITORY}/blob/main/LICENSE")
                ),
            ]
            .push(app.logs_dir.as_ref().map(|dir| {
                button(labeled(
                    &icons::FILE,
                    i18n.t("about-open-logs"),
                    Ink::GoldText,
                ))
                .padding([6, 0])
                .style(theme::link)
                .on_press(Message::OpenUrl(dir.display().to_string()))
            }))
            .spacing(4)
        ),
        card(
            column![
                text(i18n.t("about-credits-title"))
                    .font(theme::DISPLAY)
                    .size(size::TITLE),
                super::muted(i18n.t("about-credits")),
            ]
            .spacing(10)
        ),
        row![
            text(i18n.t("about-license-line"))
                .size(size::SMALL)
                .style(theme::ink(Ink::Faint)),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(14);
    centered(page, 640.0)
}
