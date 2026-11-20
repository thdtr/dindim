//! The account Settings tab. Each block has its title and explanation on the
//! left and its controls on the right.

use iced::widget::{button, column, container, row, scrollable, space, stack, text};
use iced::{Alignment, Element, Length, Padding};

use super::{divider, field, input, labeled, notice, picker, setting_row};
use crate::app::{App, Message, Picker};
use crate::icons::{self, icon};
use crate::session::OpenAccount;
use crate::theme::{self, Ink, size};
use crate::widget::popover::Popover;

fn action<'a>(label: String) -> button::Button<'a, Message> {
    button(text(label).size(size::BODY).font(theme::SANS_SEMI)).padding([12, 18])
}

pub fn view<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let s = &account.settings;

    let name = setting_row(
        i18n.t("settings-name-title"),
        i18n.t("settings-name-description"),
        row![
            input(
                &i18n.t("settings-name-placeholder"),
                &s.name,
                Message::SettingsName
            )
            .on_submit(Message::SaveName)
            .width(Length::Fill),
            action(i18n.t("action-save"))
                .style(theme::primary)
                .on_press_maybe((s.name.trim() != account.info.name).then_some(Message::SaveName)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    );

    let (badge_icon, badge_text, badge_ink) = if account.info.protected {
        (&icons::LOCK, i18n.t("settings-password-on"), Ink::Income)
    } else {
        (
            &icons::ALERT,
            i18n.t("settings-password-off"),
            Ink::GoldText,
        )
    };
    let mut password_buttons = row![
        action(i18n.t(if account.info.protected {
            "settings-password-change"
        } else {
            "settings-password-set"
        }))
        .style(theme::primary)
        .on_press(Message::SavePassword)
    ]
    .spacing(10);
    if account.info.protected {
        password_buttons = password_buttons.push(
            action(i18n.t("settings-password-remove"))
                .style(theme::secondary)
                .on_press(Message::RemovePassword),
        );
    }
    let password = setting_row(
        i18n.t("settings-password-title"),
        i18n.t("settings-password-description"),
        column![
            container(
                row![
                    icon(badge_icon, 17.0, badge_ink),
                    text(badge_text).size(size::SMALL)
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .padding([9, 12])
            .style(theme::tinted(Some(badge_ink))),
            row![
                field(
                    i18n.t("settings-password-new"),
                    input("", &s.password, Message::SettingsPassword).secure(true),
                )
                .width(Length::Fill),
                field(
                    i18n.t("settings-password-repeat"),
                    input("", &s.password_again, Message::SettingsPasswordAgain)
                        .secure(true)
                        .on_submit(Message::SavePassword),
                )
                .width(Length::Fill),
            ]
            .spacing(12),
            password_buttons,
        ]
        .spacing(14),
    );

    let open = app.picker == Some(Picker::OpeningDate);
    let date = s.opening_date.unwrap_or(app.today);
    let date_button = button(
        row![
            icon(&icons::CALENDAR, 18.0, Ink::Secondary),
            text(i18n.date_numeric(date))
                .size(size::BODY)
                .font(theme::NUM),
            space::horizontal(),
            icon(&icons::CHEVRON_DOWN, 16.0, Ink::Secondary),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([12, 14])
    .style(theme::secondary)
    .on_press(Message::TogglePicker(Picker::OpeningDate));
    let calendar = open.then(|| {
        picker::calendar(
            i18n,
            app.picker_view,
            date,
            app.today,
            Message::OpeningDate,
            Some(
                button(
                    text(i18n.t("date-today"))
                        .size(size::SMALL)
                        .font(theme::SANS_SEMI),
                )
                .padding([7, 14])
                .style(theme::secondary)
                .on_press(Message::OpeningDate(app.today))
                .into(),
            ),
        )
    });
    let mut opening_buttons = row![
        action(i18n.t("settings-opening-save"))
            .style(theme::primary)
            .on_press(Message::SaveOpening)
    ]
    .spacing(10);
    if account.opening.is_some() {
        opening_buttons = opening_buttons.push(
            action(i18n.t("action-remove"))
                .style(theme::secondary)
                .on_press(Message::ClearOpening),
        );
    }
    let symbol = i18n.currency_symbol(account.currency);
    let opening = setting_row(
        i18n.t("settings-opening-title"),
        i18n.t("settings-opening-description"),
        column![
            row![
                field(
                    i18n.t("form-amount"),
                    stack![
                        input(
                            &i18n.amount(dindim_core::Money::ZERO, account.currency),
                            &s.opening_amount,
                            Message::OpeningAmount
                        )
                        .on_submit(Message::SaveOpening)
                        .padding(Padding {
                            top: 12.0,
                            right: 14.0,
                            bottom: 12.0,
                            left: 22.0 + 10.0 * symbol.chars().count() as f32,
                        })
                        .font(theme::NUM),
                        container(
                            text(symbol)
                                .font(theme::SANS_SEMI)
                                .style(theme::ink(Ink::Secondary))
                        )
                        .padding([0, 14])
                        .center_y(Length::Fill),
                    ]
                )
                .width(Length::Fill),
                field(
                    i18n.t("settings-opening-date"),
                    Popover::new(date_button, calendar, Message::ClosePicker)
                )
                .width(Length::Fill),
            ]
            .spacing(12),
            opening_buttons,
        ]
        .spacing(14),
    );

    let currency = setting_row(
        i18n.t("settings-currency-title"),
        i18n.t("settings-currency-description"),
        container(
            text(format!(
                "{}  {}",
                account.currency.code,
                i18n.currency_symbol(account.currency)
            ))
            .size(size::BODY)
            .font(theme::NUM),
        )
        .padding([12, 14])
        .style(theme::sunk),
    );

    let csv = setting_row(
        i18n.t("settings-csv-title"),
        i18n.t("settings-csv-description"),
        row![
            button(labeled(
                &icons::DOWNLOAD,
                i18n.t("settings-csv-export"),
                Ink::Primary
            ))
            .padding([12, 18])
            .style(theme::secondary)
            .on_press(Message::ExportCsv),
            button(labeled(
                &icons::UPLOAD,
                i18n.t("settings-csv-import"),
                Ink::Primary
            ))
            .padding([12, 18])
            .style(theme::secondary)
            .on_press_maybe((!app.busy).then_some(Message::ImportCsv)),
        ]
        .spacing(10),
    );

    let remove_label = if s.confirm_remove {
        i18n.t("settings-remove-confirm")
    } else {
        i18n.t("settings-remove")
    };
    let danger = setting_row(
        i18n.t("settings-close-title"),
        i18n.t("settings-close-description"),
        row![
            button(labeled(
                &icons::LOG_OUT,
                i18n.t("settings-close"),
                Ink::Secondary
            ))
            .padding([12, 18])
            .style(theme::secondary)
            .on_press(Message::CloseAccount),
            button(labeled(&icons::TRASH, remove_label, Ink::Expense))
                .padding([12, 18])
                .style(theme::danger)
                .on_press(Message::RemoveAccount),
        ]
        .spacing(10),
    );

    let mut page = column![].spacing(16);
    if let Some(n) = notice(&s.notice) {
        page = page.push(n);
    }
    page = page.push(
        container(
            column![
                name,
                divider(),
                password,
                divider(),
                opening,
                divider(),
                currency,
                divider(),
                csv,
                divider(),
                danger
            ]
            .spacing(28),
        )
        .padding(28)
        .max_width(1080)
        .style(theme::card),
    );

    scrollable(container(page).padding(Padding {
        top: 20.0,
        right: 32.0,
        bottom: 40.0,
        left: 32.0,
    }))
    .height(Length::Fill)
    .style(theme::scroll)
    .into()
}
