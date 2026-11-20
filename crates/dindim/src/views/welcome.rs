//! The start screen: the account list and the form for a new account.

use dindim_core::Currency;
use iced::widget::{Id, button, column, container, pick_list, row, space, text};
use iced::{Alignment, Element, Length};

use super::{
    Choice, avatar, card, centered, divider, error_line, field, input, notice, section_title,
    wide_button, wordmark,
};
use crate::app::{App, Message, ids};
use crate::icons::{self, icon};
use crate::theme::{self, Ink, size};

pub fn view(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let form = &app.new_account;
    let mut page = column![
        wordmark(52.0, 40.0, false),
        space().height(4),
        text(i18n.t("welcome-tagline"))
            .size(size::LEAD)
            .style(theme::ink(Ink::Secondary)),
    ]
    .spacing(14);

    if let Some(n) = notice(&app.notice) {
        page = page.push(n);
    }

    if !app.accounts.is_empty() {
        let mut list = column![].spacing(2);
        for (i, info) in app.accounts.iter().enumerate() {
            if i > 0 {
                list = list.push(divider());
            }
            let hint: Element<'_, Message> = if info.protected {
                row![
                    icon(&icons::LOCK, 15.0, Ink::Secondary),
                    super::muted(i18n.t("welcome-needs-password"))
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
            } else {
                icon(&icons::CHEVRON_RIGHT, 18.0, Ink::Secondary).into()
            };
            list = list.push(
                button(
                    row![
                        avatar(&info.name, &info.id, 36.0),
                        text(&info.name).size(size::LEAD).font(theme::SANS_SEMI),
                        space::horizontal(),
                        hint,
                    ]
                    .spacing(14)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([10, 12])
                .style(theme::list_row(false))
                .on_press(Message::SelectAccount(info.id.clone())),
            );
        }
        page = page.push(space().height(8)).push(
            card(
                column![
                    container(section_title(i18n.t("welcome-your-accounts"))).padding([0, 8]),
                    list
                ]
                .spacing(12),
            )
            .padding([20, 16]),
        );
    }

    let currencies: Vec<Choice<Currency>> = Currency::ALL
        .iter()
        .map(|c| Choice {
            value: *c,
            label: format!("{}  {}", c.code, i18n.currency_symbol(*c)),
        })
        .collect();
    let chosen = form
        .currency
        .unwrap_or_else(|| i18n.locale().default_currency());
    let selected = currencies.iter().find(|c| c.value == chosen).cloned();

    let mut create = column![
        column![
            section_title(i18n.t("welcome-new-account")),
            super::muted(i18n.t("welcome-new-account-hint")),
        ]
        .spacing(4),
        row![
            field(
                i18n.t("welcome-name"),
                input(
                    &i18n.t("welcome-name-placeholder"),
                    &form.name,
                    Message::NewName
                )
                .id(Id::new(ids::NEW_ACCOUNT))
                .on_submit(Message::CreateAccount),
            )
            .width(Length::FillPortion(3)),
            field(
                i18n.t("welcome-currency"),
                pick_list(currencies, selected, |c| Message::NewCurrency(c.value))
                    .padding([12, 14])
                    .text_size(size::BODY)
                    .width(Length::Fill)
                    .style(theme::pick)
                    .menu_style(theme::pick_menu),
            )
            .width(Length::FillPortion(2)),
        ]
        .spacing(12),
        row![
            field(
                i18n.t("welcome-password"),
                input(
                    &i18n.t("welcome-password-placeholder"),
                    &form.password,
                    Message::NewPassword
                )
                .secure(true)
                .on_submit(Message::CreateAccount),
            )
            .width(Length::Fill),
            field(
                i18n.t("welcome-password-repeat"),
                input("", &form.password_again, Message::NewPasswordAgain)
                    .secure(true)
                    .on_submit(Message::CreateAccount),
            )
            .width(Length::Fill),
        ]
        .spacing(12),
        row![
            icon(&icons::LOCK, 15.0, Ink::Secondary),
            super::muted(i18n.t("welcome-password-hint")),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(16);
    if let Some(err) = &form.error {
        create = create.push(error_line(err.clone()));
    }
    create = create.push(
        wide_button(i18n.t("welcome-create"))
            .style(theme::primary)
            .on_press(Message::CreateAccount),
    );

    page = page.push(card(create));
    centered(page, 600.0)
}
