use iced::widget::{Id, column, row, text, text_input};
use iced::{Alignment, Element};

use super::{avatar, card, centered, error_line, wide_button};
use crate::app::{App, Message, ids};
use crate::icons::{self, icon};
use crate::theme::{self, Ink, size};

pub fn view<'a>(app: &'a App, id: &'a str) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let name = app
        .accounts
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.name.as_str())
        .unwrap_or(id);
    let busy = app.unlock.busy;
    let mut body = column![
        avatar(name, id, 64.0),
        column![
            text(name)
                .font(theme::DISPLAY_BOLD)
                .size(size::HEADING + 4.0),
            row![
                icon(&icons::LOCK, 16.0, Ink::Secondary),
                text(i18n.t("unlock-prompt"))
                    .size(size::BODY)
                    .style(theme::ink(Ink::Secondary))
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(6)
        .align_x(Alignment::Center),
        text_input(&i18n.t("unlock-placeholder"), &app.unlock.password)
            .id(Id::new(ids::PASSWORD))
            .secure(true)
            .on_input_maybe((!busy).then_some(Message::UnlockPassword))
            .on_submit(Message::Unlock)
            .padding([14, 16])
            .size(size::LEAD)
            .style(theme::input),
    ]
    .spacing(18)
    .align_x(Alignment::Center);
    if let Some(err) = &app.unlock.error {
        body = body.push(error_line(err.clone()));
    }
    body = body.push(
        wide_button(i18n.t(if busy {
            "unlock-opening"
        } else {
            "unlock-open"
        }))
        .style(theme::primary)
        .on_press_maybe((!busy).then_some(Message::Unlock)),
    );
    centered(card(body).padding(32), 440.0)
}
