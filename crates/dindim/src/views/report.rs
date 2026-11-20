//! The Report tab: the month told in short sentences, big numbers and bars.

use std::sync::LazyLock;

use dindim_core::Money;
use dindim_core::report::{CategoryShare, Headline, MonthReport, SpendingChange, Tone, YearReport};
use dindim_i18n::{I18n, args};
use iced::alignment::Horizontal;
use iced::widget::svg::Handle;
use iced::widget::{Column, button, column, container, row, scrollable, space, text};
use iced::{Alignment, Element, Length, Padding};

use super::{capitalize, card, section_title};
use crate::app::{App, Message};
use crate::icons::{self, icon};
use crate::session::OpenAccount;
use crate::theme::{self, Ink, size};

pub fn view<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let money = |m: Money| i18n.money(m, account.currency);
    let r = MonthReport::build(app.month, &account.entries, account.opening);
    let month = i18n.month_name(app.month.month());
    let headline = r.headline();
    let headline_text = match headline {
        Headline::NoEntries => i18n.t_with(
            "report-headline-empty",
            &args! { "month" => month.as_str() },
        ),
        Headline::NoIncome { expenses } => i18n.t_with(
            "report-headline-no-income",
            &args! { "month" => month.as_str(), "amount" => money(expenses) },
        ),
        Headline::Surplus { amount } => i18n.t_with(
            "report-headline-surplus",
            &args! { "month" => month.as_str(), "amount" => money(amount) },
        ),
        Headline::BreakEven => {
            i18n.t_with("report-headline-even", &args! { "month" => month.as_str() })
        }
        Headline::Deficit { amount } => i18n.t_with(
            "report-headline-deficit",
            &args! { "month" => month.as_str(), "amount" => money(amount) },
        ),
    };
    let tone = match headline.tone() {
        Tone::Good => Ink::Income,
        Tone::Neutral => Ink::Primary,
        Tone::Bad => Ink::Expense,
    };

    let mut intro = column![
        text(headline_text)
            .font(theme::DISPLAY_BOLD)
            .size(34)
            .style(theme::ink(tone))
    ]
    .spacing(8);
    let mut sentences = Vec::new();
    if let Some(pct) = r.spending_ratio() {
        sentences.push(i18n.t_with("report-ratio", &args! { "percent" => pct }));
    }
    if let Some(change) = r.spending_change() {
        let previous = i18n.month_name(app.month.prev().month());
        sentences.push(match change {
            SpendingChange::More(diff) => i18n.t_with(
                "report-change-more",
                &args! { "amount" => money(diff), "month" => previous },
            ),
            SpendingChange::Less(diff) => i18n.t_with(
                "report-change-less",
                &args! { "amount" => money(diff), "month" => previous },
            ),
            SpendingChange::Same => {
                i18n.t_with("report-change-same", &args! { "month" => previous })
            }
        });
    }
    if !sentences.is_empty() {
        intro = intro.push(
            text(sentences.join("  "))
                .size(size::LEAD)
                .style(theme::ink(Ink::Secondary)),
        );
    }

    let left = r.left_over();
    let numbers = row![
        big_number(
            i18n.t("summary-in"),
            money(r.income),
            Ink::Income,
            &icons::ARROW_IN
        ),
        big_number(
            i18n.t("summary-out"),
            money(r.expenses),
            Ink::Expense,
            &icons::ARROW_OUT
        ),
        if left.is_negative() {
            big_number(
                i18n.t("summary-short"),
                money(left.abs()),
                Ink::Expense,
                &icons::ALERT,
            )
        } else {
            big_number(
                i18n.t("summary-left"),
                money(left),
                Ink::Primary,
                &icons::CHECK_CIRCLE,
            )
        },
    ]
    .spacing(16);

    let mut page = column![intro, numbers].spacing(20);

    if !r.expenses_by_category.is_empty() {
        page = page.push(card(
            column![
                section_title(i18n.t("report-where-it-went")),
                shares(i18n, account, &r.expenses_by_category, Ink::Expense, false)
            ]
            .spacing(18),
        ));
    }
    if !r.income_by_category.is_empty() {
        page = page.push(card(
            column![
                section_title(i18n.t("report-where-it-came-from")),
                shares(i18n, account, &r.income_by_category, Ink::Income, true)
            ]
            .spacing(18),
        ));
    }

    let net = r.reserve_net();
    let mut reserve_text = i18n.t_with(
        "report-reserve-balance",
        &args! { "amount" => money(r.reserve_balance) },
    );
    if net.is_positive() {
        reserve_text.push(' ');
        reserve_text
            .push_str(&i18n.t_with("report-reserve-saved", &args! { "amount" => money(net) }));
    } else if net.is_negative() {
        reserve_text.push(' ');
        reserve_text.push_str(&i18n.t_with(
            "report-reserve-taken",
            &args! { "amount" => money(net.abs()) },
        ));
    }
    let reserve = card(
        column![
            row![
                icon(&icons::COINS, 18.0, Ink::Reserve),
                text(i18n.t("report-reserve-title"))
                    .size(size::BODY)
                    .font(theme::SANS_SEMI)
                    .style(theme::ink(Ink::Reserve)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(money(r.reserve_balance))
                .font(theme::DISPLAY_BOLD)
                .size(size::HERO)
                .style(theme::ink(Ink::Reserve)),
            text(reserve_text)
                .size(size::BODY)
                .style(theme::ink(Ink::Secondary)),
        ]
        .spacing(6),
    );

    let balance_note = if account.opening.is_some() {
        i18n.t("report-balance-note")
    } else {
        i18n.t("report-balance-tip")
    };
    let balance = card(
        column![
            text(i18n.t("report-balance-title"))
                .size(size::BODY)
                .font(theme::SANS_SEMI)
                .style(theme::ink(Ink::Secondary)),
            text(money(r.balance))
                .font(theme::DISPLAY_BOLD)
                .size(size::HERO)
                .style(theme::ink(if r.balance.is_negative() {
                    Ink::Expense
                } else {
                    Ink::Primary
                })),
            super::muted(balance_note),
        ]
        .spacing(6),
    );

    page = page.push(row![reserve, balance].spacing(16));
    page = page.push(year_chart(app, account));

    scrollable(container(page).padding(Padding {
        top: 24.0,
        right: 32.0,
        bottom: 40.0,
        left: 32.0,
    }))
    .height(Length::Fill)
    .style(theme::scroll)
    .into()
}

fn big_number<'a>(
    label: String,
    value: String,
    ink: Ink,
    glyph: &'static LazyLock<Handle>,
) -> Element<'a, Message> {
    card(
        column![
            row![
                icon(glyph, 18.0, ink),
                text(label)
                    .size(size::BODY)
                    .font(theme::SANS_SEMI)
                    .style(theme::ink(Ink::Secondary)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(value)
                .font(theme::DISPLAY_BOLD)
                .size(size::HERO)
                .style(theme::ink(ink))
                .wrapping(text::Wrapping::None),
        ]
        .spacing(6),
    )
    .padding([20, 24])
    .into()
}

fn shares<'a>(
    i18n: &I18n,
    account: &OpenAccount,
    list: &[CategoryShare],
    ink: Ink,
    income: bool,
) -> Element<'a, Message> {
    let max = list.iter().map(|s| s.total).max().unwrap_or(Money::ZERO);
    let mut col = Column::new().spacing(14);
    for share in list {
        let name = match account.category_name(share.category) {
            Some(name) => name.to_string(),
            None => i18n.t(if income {
                "report-uncategorized-income"
            } else {
                "report-uncategorized"
            }),
        };
        let filled = portion(share.total, max).max(4);
        let bar = container(row![
            container(space().width(Length::Fill).height(12))
                .width(Length::FillPortion(filled))
                .style(theme::fill(ink, 6.0)),
            space().width(Length::FillPortion(1000 - filled.min(999))),
        ])
        .width(Length::Fill)
        .style(theme::fill_sunk(6.0));
        col = col.push(
            row![
                text(name)
                    .size(size::BODY)
                    .font(theme::SANS_SEMI)
                    .width(200),
                bar,
                text(i18n.money(share.total, account.currency))
                    .size(size::BODY)
                    .font(theme::NUM)
                    .width(150)
                    .align_x(Horizontal::Right),
                text(format!("{}%", share.percent))
                    .size(size::SMALL)
                    .font(theme::NUM)
                    .style(theme::ink(Ink::Secondary))
                    .width(48)
                    .align_x(Horizontal::Right),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        );
    }
    col.into()
}

/// How much of `max` (0 to 1000) `value` represents.
fn portion(value: Money, max: Money) -> u16 {
    if !max.is_positive() || !value.is_positive() {
        return 0;
    }
    ((i128::from(value.minor()) * 1000) / i128::from(max.minor())).clamp(0, 1000) as u16
}

fn vertical_bar<'a>(value: Money, peak: Money, ink: Ink) -> Element<'a, Message> {
    let filled = portion(value, peak);
    let mut col = Column::new().width(14).height(Length::Fill);
    if filled < 1000 {
        col = col.push(space().height(Length::FillPortion(1000 - filled)));
    }
    if filled > 0 {
        col = col.push(
            container(space().width(14).height(Length::Fill))
                .height(Length::FillPortion(filled))
                .style(theme::fill(ink, 4.0)),
        );
    }
    col.into()
}

fn legend_dot<'a>(ink: Ink) -> Element<'a, Message> {
    container(space().width(10).height(10))
        .style(theme::fill(ink, 3.0))
        .into()
}

fn year_chart<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let year = YearReport::build(app.month.year(), &account.entries);
    let peak = year.peak();
    let mut bars = row![].spacing(6).height(180);
    for m in &year.months {
        let selected = m.month == app.month;
        let label = text(capitalize(&i18n.month_short(m.month.month())))
            .size(size::CAPTION)
            .font(theme::SANS_SEMI)
            .style(theme::ink(if selected {
                Ink::Primary
            } else {
                Ink::Secondary
            }));
        let column = column![
            container(
                row![
                    vertical_bar(m.income, peak, Ink::Income),
                    vertical_bar(m.expenses, peak, Ink::Expense),
                ]
                .spacing(3),
            )
            .center_x(Length::Fill)
            .height(Length::Fill),
            label,
        ]
        .spacing(8)
        .align_x(Alignment::Center);
        bars = bars.push(
            button(column)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([8, 2])
                .style(theme::list_row(selected))
                .on_press(Message::GoToMonth(m.month)),
        );
    }

    let legend = row![
        legend_dot(Ink::Income),
        text(i18n.t_with(
            "report-year-in",
            &args! { "amount" => i18n.money(year.income(), account.currency) },
        ))
        .size(size::SMALL),
        space().width(18),
        legend_dot(Ink::Expense),
        text(i18n.t_with(
            "report-year-out",
            &args! { "amount" => i18n.money(year.expenses(), account.currency) },
        ))
        .size(size::SMALL),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    card(
        column![
            row![
                text(i18n.t_with(
                    "report-year-title",
                    &args! { "year" => year.year.to_string() }
                ))
                .font(theme::DISPLAY)
                .size(size::TITLE),
                space::horizontal(),
                super::muted(i18n.t("report-year-hint")),
            ]
            .align_y(Alignment::Center),
            bars,
            legend,
        ]
        .spacing(16),
    )
    .into()
}
