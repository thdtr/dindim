//! The Entries tab: the form on the left, the month summary and the entries
//! grouped by day on the right.

use dindim_core::report::MonthReport;
use dindim_core::search::EntryQuery;
use dindim_core::{Entry, Kind, Money};
use dindim_i18n::args;
use iced::alignment::Horizontal;
use iced::widget::{
    Column, Id, button, column, container, responsive, row, scrollable, space, stack, text,
    text_editor, text_input,
};
use iced::{Alignment, Element, Length, Padding};

use super::{
    Segment, capitalize, divider, error_line, field, kind_icon, kind_ink, label, net_amount,
    picker, segmented, signed_amount, vrule,
};
use crate::app::{App, Message, Picker, ids};
use crate::icons::{self, icon};
use crate::session::OpenAccount;
use crate::theme::{self, Ink, size};
use crate::widget::popover::Popover;

const PANEL_WIDTH: f32 = 344.0;
const MENU_MAX_HEIGHT: f32 = 264.0;

pub fn view<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let panel = scrollable(form(app, account))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(4)
                .scroller_width(4)
                .spacing(6),
        ))
        .height(Length::Fill)
        .style(theme::scroll);
    row![
        container(panel).width(PANEL_WIDTH),
        column![summary(app, account), list(app, account)]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(24)
    .padding(Padding {
        top: 20.0,
        right: 32.0,
        bottom: 24.0,
        left: 32.0,
    })
    .height(Length::Fill)
    .into()
}

fn form<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let f = &account.form;
    let accent = kind_ink(f.kind);
    let editing = f.editing.is_some();

    let head = editing.then(|| {
        row![
            text(i18n.t("form-edit-title"))
                .font(theme::DISPLAY)
                .size(size::TITLE),
            space::horizontal(),
            super::icon_button(&icons::X, Message::CancelEdit),
        ]
        .height(36)
        .align_y(Alignment::Center)
    });

    let reserve_kind = if f.kind.is_reserve() {
        f.kind
    } else {
        Kind::ReserveDeposit
    };
    let kinds = segmented(vec![
        Segment::new(
            i18n.t("kind-expense"),
            f.kind == Kind::Expense,
            Ink::Expense,
            Message::SetKind(Kind::Expense),
        )
        .icon(&icons::ARROW_OUT),
        Segment::new(
            i18n.t("kind-income"),
            f.kind == Kind::Income,
            Ink::Income,
            Message::SetKind(Kind::Income),
        )
        .icon(&icons::ARROW_IN),
        Segment::new(
            i18n.t("kind-reserve"),
            f.kind.is_reserve(),
            Ink::Reserve,
            Message::SetKind(reserve_kind),
        )
        .icon(&icons::COINS),
    ]);

    let symbol = i18n.currency_symbol(account.currency);
    let placeholder = i18n.amount(Money::ZERO, account.currency);
    let amount = stack![
        text_input(&placeholder, &f.amount)
            .id(Id::new(ids::AMOUNT))
            .on_input(Message::Amount)
            .on_submit(Message::SubmitEntry)
            .padding(Padding {
                top: 10.0,
                right: 14.0,
                bottom: 10.0,
                left: 24.0 + 11.0 * symbol.chars().count() as f32,
            })
            .size(28)
            .font(theme::NUM_BOLD)
            .style(theme::input),
        container(
            text(symbol)
                .size(size::LEAD)
                .font(theme::SANS_SEMI)
                .style(theme::ink(Ink::Secondary))
        )
        .padding([0, 16])
        .center_y(Length::Fill),
    ];

    // Category for expenses and income, direction for the emergency fund.
    // Both have the same height so the form doesn't jump.
    let slot: Element<'a, Message> = if f.kind.category_kind().is_some() {
        field(i18n.t("form-category"), category_field(app, account)).into()
    } else {
        field(
            i18n.t("form-movement"),
            segmented(vec![
                Segment::new(
                    i18n.t("reserve-deposit"),
                    f.kind == Kind::ReserveDeposit,
                    Ink::Reserve,
                    Message::SetKind(Kind::ReserveDeposit),
                )
                .icon(&icons::ARROW_IN),
                Segment::new(
                    i18n.t("reserve-withdrawal"),
                    f.kind == Kind::ReserveWithdrawal,
                    Ink::Reserve,
                    Message::SetKind(Kind::ReserveWithdrawal),
                )
                .icon(&icons::ARROW_OUT),
            ]),
        )
        .into()
    };

    let date = field(i18n.t("form-date"), date_control(app, account));

    let note = column![
        row![
            label(i18n.t("form-note")),
            text(i18n.t("form-optional"))
                .size(size::CAPTION)
                .style(theme::ink(Ink::Faint))
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        text_editor(&account.note_editor)
            .id(Id::new(ids::NOTE))
            .placeholder(i18n.t("form-note-placeholder"))
            .on_action(Message::Note)
            .key_binding(|press| {
                let enter = matches!(
                    press.key.as_ref(),
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                );
                if enter && !press.modifiers.shift() {
                    Some(text_editor::Binding::Custom(Message::SubmitEntry))
                } else {
                    text_editor::Binding::from_key_press(press)
                }
            })
            .padding([12, 14])
            .size(size::BODY)
            .style(theme::editor),
    ]
    .spacing(7);

    let submit_label = match (editing, f.kind) {
        (true, _) => i18n.t("form-save-changes"),
        (false, Kind::Expense) => i18n.t("form-add-expense"),
        (false, Kind::Income) => i18n.t("form-add-income"),
        (false, Kind::ReserveDeposit) => i18n.t("form-add-deposit"),
        (false, Kind::ReserveWithdrawal) => i18n.t("form-add-withdrawal"),
    };
    let submit = button(
        container(text(submit_label).size(size::BODY).font(theme::SANS_BOLD))
            .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .padding([13, 16])
    .style(theme::solid(accent))
    .on_press(Message::SubmitEntry);

    let mut body = Column::new()
        .push(head)
        .push(kinds)
        .push(field(i18n.t("form-amount"), amount))
        .push(slot)
        .push(date)
        .push(note)
        .spacing(16);
    if let Some(err) = &f.error {
        body = body.push(error_line(err.message(i18n)));
    }
    body = body.push(submit);

    if editing {
        let delete_label = if f.confirm_delete {
            i18n.t("form-confirm-delete")
        } else {
            i18n.t("form-delete")
        };
        body = body.push(
            row![
                button(
                    container(text(i18n.t("action-cancel")).font(theme::SANS_SEMI))
                        .center_x(Length::Fill)
                )
                .width(Length::Fill)
                .padding([10, 12])
                .style(theme::secondary)
                .on_press(Message::CancelEdit),
                button(
                    container(super::labeled(&icons::TRASH, delete_label, Ink::Expense))
                        .center_x(Length::Fill)
                )
                .width(Length::Fill)
                .padding([10, 12])
                .style(theme::danger)
                .on_press(Message::DeleteEntry),
            ]
            .spacing(8),
        );
    } else {
        body = body.push(
            text(i18n.t("form-keyboard-hint"))
                .size(12)
                .style(theme::ink(Ink::Faint))
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        );
    }

    container(body)
        .padding(22)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

/// A text field that filters the account's categories as you type, with a
/// menu that can also create a new category on the spot.
fn category_field<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let f = &account.form;
    let shown = match account.category(f.category) {
        Some(c) if f.category_query.is_empty() => c.name.clone(),
        _ => f.category_query.clone(),
    };
    let placeholder = if account.form.options(&account.categories).is_empty() {
        i18n.t("form-category-placeholder-empty")
    } else {
        i18n.t("form-category-placeholder")
    };
    let input = stack![
        text_input(&placeholder, &shown)
            .id(Id::new(ids::CATEGORY))
            .on_input(Message::CategoryQuery)
            .on_submit(Message::CategorySubmit)
            .padding(Padding {
                top: 12.0,
                right: 44.0,
                bottom: 12.0,
                left: 14.0,
            })
            .size(size::BODY)
            .style(theme::input),
        container(
            button(container(icon(&icons::CHEVRON_DOWN, 18.0, Ink::Secondary)).center(30))
                .padding(0)
                .style(theme::ghost)
                .on_press(Message::ToggleCategoryMenu)
        )
        .align_right(Length::Fill)
        .center_y(Length::Fill)
        .padding([0, 6]),
    ];

    let popup = account.category_menu.map(|highlight| {
        let (items, create) = account.menu_rows();
        let mut rows = Column::new().spacing(2);
        for (index, category) in items.iter().enumerate() {
            rows = rows.push(
                button(text(category.name.clone()).size(size::BODY))
                    .width(Length::Fill)
                    .padding([8, 10])
                    .style(theme::menu_item(index == highlight))
                    .on_press(Message::PickCategory(category.id)),
            );
        }
        if create {
            let label = i18n.t_with(
                "form-create-category",
                &args! { "name" => f.category_query.trim() },
            );
            rows = rows.push(
                button(
                    row![
                        icon(&icons::PLUS, 16.0, Ink::GoldText),
                        text(label).size(size::BODY).font(theme::SANS_SEMI),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([8, 10])
                .style(theme::menu_item(highlight == items.len()))
                .on_press(Message::CreateCategoryInline),
            );
        }
        if items.is_empty() && !create {
            rows = rows
                .push(container(super::muted(i18n.t("form-category-menu-empty"))).padding([8, 10]));
        }
        container(scrollable(rows).height(Length::Shrink).style(theme::scroll))
            .padding(6)
            .max_height(MENU_MAX_HEIGHT)
            .width(Length::Fill)
            .style(theme::popover)
            .into()
    });

    Popover::new(input, popup, Message::CloseCategoryMenu)
        .match_width()
        .into()
}

/// [Today | Yesterday | Other day], the last one opening a calendar.
fn date_control<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let today = app.today;
    let yesterday = today.yesterday().unwrap_or(today);
    let date = account.form.date_or(today);
    let other = date != today && date != yesterday;
    let open = app.picker == Some(Picker::EntryDate);
    let calendar = open
        .then(|| picker::calendar(i18n, app.picker_view, date, today, Message::EntryDate, None));

    segmented(vec![
        Segment::new(
            i18n.t("date-today"),
            date == today,
            Ink::Primary,
            Message::EntryDate(today),
        )
        .portion(4),
        Segment::new(
            i18n.t("date-yesterday"),
            date == yesterday,
            Ink::Primary,
            Message::EntryDate(yesterday),
        )
        .portion(4),
        Segment::new(
            if other {
                i18n.date_relative(date, today)
            } else {
                i18n.t("form-other-day")
            },
            other || open,
            Ink::Primary,
            Message::TogglePicker(Picker::EntryDate),
        )
        .icon(&icons::CALENDAR)
        .portion(5)
        .wrap(move |b| {
            Popover::new(b, calendar, Message::ClosePicker)
                .align_end()
                .into()
        }),
    ])
}

fn stat<'a>(title: String, value: String, ink: Ink, value_size: f32) -> Element<'a, Message> {
    column![
        text(title)
            .size(size::CAPTION)
            .font(theme::SANS_SEMI)
            .style(theme::ink(Ink::Secondary)),
        text(value)
            .font(theme::DISPLAY_BOLD)
            .size(value_size)
            .style(theme::ink(ink))
            .wrapping(text::Wrapping::None),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

/// In, out and left over. The numbers shrink with the window so they never wrap.
fn summary<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let r = MonthReport::build(app.month, &account.entries, account.opening);
    let left = r.left_over();
    let money = |m: Money| i18n.money(m, account.currency);
    let stats = [
        (i18n.t("summary-in"), money(r.income), Ink::Income),
        (i18n.t("summary-out"), money(r.expenses), Ink::Expense),
        if left.is_negative() {
            (i18n.t("summary-short"), money(left.abs()), Ink::Expense)
        } else {
            (i18n.t("summary-left"), money(left), Ink::Primary)
        },
    ];
    let line = responsive(move |space| {
        let per_stat = (space.width - 44.0 - 2.0 * 33.0) / 3.0;
        let value_size = if per_stat >= 170.0 {
            26.0
        } else if per_stat >= 140.0 {
            22.0
        } else {
            19.0
        };
        let [a, b, c] = stats.clone();
        container(
            row![
                stat(a.0, a.1, a.2, value_size),
                vrule(40.0),
                stat(b.0, b.1, b.2, value_size),
                vrule(40.0),
                stat(c.0, c.1, c.2, value_size),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        )
        .padding([0, 22])
        .center_y(Length::Fill)
        .into()
    });
    container(line)
        .height(86)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

fn list<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let searching = !account.search.trim().is_empty();
    let shown: Vec<&Entry> = if searching {
        let amount = i18n.parse_amount(&account.search, account.currency).ok();
        let query = EntryQuery::new(&account.search, amount);
        account
            .entries
            .iter()
            .filter(|e| query.matches(e, account.category_name(e.category_id)))
            .collect()
    } else {
        account
            .entries
            .iter()
            .filter(|e| app.month.contains(e.date))
            .collect()
    };

    let title = if searching {
        i18n.t("list-search-results")
    } else {
        i18n.t_with(
            "list-title",
            &args! { "month" => i18n.month_name(app.month.month()) },
        )
    };
    let count = i18n.t_with("list-count", &args! { "count" => shown.len() });
    let reserve_net = shown
        .iter()
        .map(|e| e.kind.reserve_effect(e.amount))
        .sum::<Money>();
    let reserve_note = (!reserve_net.is_zero() && !searching).then(|| {
        let key = if reserve_net.is_negative() {
            "list-reserve-out"
        } else {
            "list-reserve-in"
        };
        let amount = i18n.money(reserve_net.abs(), account.currency);
        row![
            icon(&icons::COINS, 15.0, Ink::Reserve),
            text(i18n.t_with(key, &args! { "amount" => amount }))
                .size(size::SMALL)
                .font(theme::SANS_SEMI)
                .style(theme::ink(Ink::Reserve)),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    });

    let search = stack![
        text_input(&i18n.t("list-search-placeholder"), &account.search)
            .id(Id::new(ids::ENTRY_SEARCH))
            .on_input(Message::Search)
            .padding(Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 36.0,
            })
            .size(size::SMALL)
            .style(theme::input),
        container(icon(&icons::SEARCH, 16.0, Ink::Faint))
            .padding([0, 12])
            .center_y(Length::Fill),
    ];

    let head = container(
        column![
            row![
                text(capitalize(&title))
                    .font(theme::DISPLAY)
                    .size(size::LEAD)
                    .wrapping(text::Wrapping::None),
                text(count)
                    .size(size::SMALL)
                    .style(theme::ink(Ink::Secondary))
                    .wrapping(text::Wrapping::None),
                space::horizontal(),
                container(search).width(240),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        ]
        .push(reserve_note)
        .spacing(6),
    )
    .padding([12, 22]);

    let body: Element<'a, Message> = if shown.is_empty() {
        let (title, hint) = if searching {
            (
                i18n.t("list-search-empty"),
                i18n.t("list-search-empty-hint"),
            )
        } else {
            (
                i18n.t_with(
                    "list-empty",
                    &args! { "month" => i18n.month_name(app.month.month()) },
                ),
                i18n.t("list-empty-hint"),
            )
        };
        container(
            column![
                icons::mark(44.0, Ink::LineStrong),
                text(title).font(theme::DISPLAY).size(size::LEAD),
                super::muted(hint),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .center(Length::Fill)
        .into()
    } else {
        let mut col = Column::new().spacing(2);
        let mut days = shown.chunk_by(|a, b| a.date == b.date).peekable();
        while let Some(day) = days.next() {
            let date = day[0].date;
            let net: Money = day.iter().map(|e| e.kind.cash_effect(e.amount)).sum();
            col = col.push(
                container(
                    row![
                        text(capitalize(&i18n.date_relative(date, app.today)))
                            .size(size::CAPTION)
                            .font(theme::SANS_SEMI)
                            .style(theme::ink(Ink::Secondary)),
                        space::horizontal(),
                        text(net_amount(i18n, account, net))
                            .size(size::CAPTION)
                            .font(theme::NUM)
                            .style(theme::ink(Ink::Secondary)),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(Padding {
                    top: 6.0,
                    right: 12.0,
                    bottom: 4.0,
                    left: 12.0,
                }),
            );
            for entry in day {
                col = col.push(entry_row(app, account, entry));
            }
            if days.peek().is_some() {
                col = col.push(space().height(12));
            }
        }
        scrollable(container(col).padding(Padding {
            top: 8.0,
            right: 14.0,
            bottom: 14.0,
            left: 10.0,
        }))
        .height(Length::Fill)
        .style(theme::scroll)
        .into()
    };

    container(column![head, divider(), body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card)
        .into()
}

fn entry_row<'a>(app: &'a App, account: &'a OpenAccount, entry: &'a Entry) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let accent = kind_ink(entry.kind);
    let title = match entry.kind {
        Kind::ReserveDeposit => Some(i18n.t("entry-reserve-deposit")),
        Kind::ReserveWithdrawal => Some(i18n.t("entry-reserve-withdrawal")),
        _ => account.category_name(entry.category_id).map(str::to_string),
    };
    let title = match title {
        Some(name) => text(name).size(size::BODY).font(theme::SANS_SEMI),
        None => text(i18n.t(match entry.kind {
            Kind::Income => "entry-income-uncategorized",
            _ => "entry-expense-uncategorized",
        }))
        .size(size::BODY)
        .font(theme::SANS_SEMI)
        .style(theme::ink(Ink::Secondary)),
    };
    let mut texts = column![title].spacing(1);
    if !entry.note.is_empty() {
        texts = texts.push(super::muted(entry.note.clone()));
    }
    let badge = container(icon(kind_icon(entry.kind), 18.0, accent))
        .center(36)
        .style(theme::fill_tint(accent, 10.0));
    let line = row![
        badge,
        texts.width(Length::Fill),
        text(signed_amount(i18n, account, entry.kind, entry.amount))
            .size(size::BODY)
            .font(theme::NUM)
            .style(theme::ink(accent)),
    ]
    .spacing(14)
    .align_y(Alignment::Center);
    button(line)
        .width(Length::Fill)
        .padding([8, 12])
        .style(theme::list_row(account.form.editing == Some(entry.id)))
        .on_press(Message::EditEntry(entry.id))
        .into()
}
