//! The Categories tab: one list at a time (expenses or income), a search box
//! that also creates, how much each category is used and actions per row.

use std::collections::HashMap;

use dindim_core::text::fold;
use dindim_core::{Category, CategoryId, CategoryKind, Money};
use dindim_i18n::args;
use iced::alignment::Horizontal;
use iced::widget::{
    Column, Id, button, column, container, row, scrollable, space, stack, text, text_input,
};
use iced::{Alignment, Element, Length, Padding};

use super::{Segment, divider, notice, segmented};
use crate::app::{App, Message, ids};
use crate::icons::{self, icon};
use crate::session::OpenAccount;
use crate::theme::{self, Ink, size};

const COUNT_WIDTH: f32 = 120.0;
const TOTAL_WIDTH: f32 = 160.0;
const ACTIONS_WIDTH: f32 = 210.0;

#[derive(Default, Clone, Copy)]
struct Usage {
    count: usize,
    year_total: Money,
}

pub fn view<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let m = &account.manager;
    let year = app.today.year();

    let mut usage: HashMap<CategoryId, Usage> = HashMap::new();
    for e in &account.entries {
        if let Some(id) = e.category_id {
            let u = usage.entry(id).or_default();
            u.count += 1;
            if e.date.year() == year {
                u.year_total += e.amount;
            }
        }
    }

    let count_of = |kind| account.categories.iter().filter(|c| c.kind == kind).count();
    let switch = container(segmented(vec![
        Segment::new(
            i18n.t_with(
                "categories-expenses",
                &args! { "count" => count_of(CategoryKind::Expense) },
            ),
            m.kind == CategoryKind::Expense,
            Ink::Expense,
            Message::ManagerKind(CategoryKind::Expense),
        )
        .icon(&icons::ARROW_OUT),
        Segment::new(
            i18n.t_with(
                "categories-income",
                &args! { "count" => count_of(CategoryKind::Income) },
            ),
            m.kind == CategoryKind::Income,
            Ink::Income,
            Message::ManagerKind(CategoryKind::Income),
        )
        .icon(&icons::ARROW_IN),
    ]))
    .width(340);

    let mut top = column![
        row![
            switch,
            space::horizontal(),
            super::muted(i18n.t("categories-delete-note")),
        ]
        .spacing(24)
        .align_y(Alignment::Center)
    ]
    .spacing(14);
    if let Some(n) = notice(&m.notice) {
        top = top.push(n);
    }

    let query = fold(&m.search);
    let mut list: Vec<&Category> = account
        .categories
        .iter()
        .filter(|c| c.kind == m.kind)
        .filter(|c| query.is_empty() || fold(&c.name).contains(&query))
        .collect();
    list.sort_by_cached_key(|c| fold(&c.name));
    let exists = account
        .categories
        .iter()
        .any(|c| c.kind == m.kind && fold(&c.name) == query);
    let can_create = !query.is_empty() && !exists;

    let placeholder = i18n.t(match m.kind {
        CategoryKind::Expense => "categories-search-expense",
        CategoryKind::Income => "categories-search-income",
    });
    let search = stack![
        text_input(&placeholder, &m.search)
            .id(Id::new(ids::CATEGORY_SEARCH))
            .on_input(Message::ManagerSearch)
            .on_submit(Message::CreateCategory)
            .padding(Padding {
                top: 12.0,
                right: 14.0,
                bottom: 12.0,
                left: 42.0,
            })
            .size(size::BODY)
            .style(theme::input),
        container(icon(&icons::SEARCH, 18.0, Ink::Secondary))
            .padding([0, 14])
            .center_y(Length::Fill),
    ];
    let create_label = if can_create {
        i18n.t_with(
            "categories-create-named",
            &args! { "name" => m.search.trim() },
        )
    } else {
        i18n.t("categories-create")
    };
    let toolbar = row![
        container(search).width(Length::Fill),
        button(
            row![
                icon(&icons::PLUS, 18.0, Ink::OnGold),
                text(create_label)
                    .size(size::BODY)
                    .font(theme::SANS_SEMI)
                    .wrapping(text::Wrapping::None)
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .padding([12, 18])
        .style(theme::primary)
        .on_press_maybe(can_create.then_some(Message::CreateCategory)),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let col_head = |title: String, width: Length, right: bool| {
        text(title)
            .size(size::CAPTION)
            .font(theme::SANS_SEMI)
            .style(theme::ink(Ink::Secondary))
            .width(width)
            .align_x(if right {
                Horizontal::Right
            } else {
                Horizontal::Left
            })
    };
    let table_head = container(
        row![
            col_head(i18n.t("categories-column-name"), Length::Fill, false),
            col_head(
                i18n.t("categories-column-entries"),
                Length::Fixed(COUNT_WIDTH),
                true
            ),
            col_head(
                i18n.t_with(
                    "categories-column-total",
                    &args! { "year" => year.to_string() }
                ),
                Length::Fixed(TOTAL_WIDTH),
                true
            ),
            space().width(ACTIONS_WIDTH),
        ]
        .spacing(16),
    )
    .padding(Padding {
        top: 10.0,
        right: 24.0,
        bottom: 10.0,
        left: 42.0,
    });

    let body: Element<'a, Message> = if list.is_empty() {
        let message = if query.is_empty() {
            i18n.t("categories-empty")
        } else {
            i18n.t_with("categories-no-match", &args! { "name" => m.search.trim() })
        };
        container(
            column![
                icon(&icons::TAG, 32.0, Ink::LineStrong),
                text(message)
                    .size(size::BODY)
                    .style(theme::ink(Ink::Secondary)),
            ]
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .center(Length::Fill)
        .into()
    } else {
        let mut rows = Column::new().spacing(2);
        for c in list {
            let u = usage.get(&c.id).copied().unwrap_or_default();
            rows = rows.push(category_row(app, account, c, u));
        }
        scrollable(container(rows).padding(Padding {
            top: 6.0,
            right: 12.0,
            bottom: 12.0,
            left: 10.0,
        }))
        .height(Length::Fill)
        .style(theme::scroll)
        .into()
    };

    let table = container(column![
        container(toolbar).padding(Padding {
            top: 18.0,
            right: 22.0,
            bottom: 12.0,
            left: 22.0,
        }),
        table_head,
        divider(),
        body,
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::card);

    column![top, table]
        .spacing(16)
        .padding(Padding {
            top: 20.0,
            right: 32.0,
            bottom: 24.0,
            left: 32.0,
        })
        .height(Length::Fill)
        .into()
}

fn small_button<'a>(label: String) -> button::Button<'a, Message> {
    button(text(label).size(size::SMALL).font(theme::SANS_SEMI)).padding([8, 14])
}

fn category_row<'a>(
    app: &'a App,
    account: &'a OpenAccount,
    c: &'a Category,
    u: Usage,
) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let m = &account.manager;
    let accent = match c.kind {
        CategoryKind::Expense => Ink::Expense,
        CategoryKind::Income => Ink::Income,
    };

    if m.confirm_delete == Some(c.id) {
        return container(
            row![
                icon(&icons::ALERT, 18.0, Ink::Expense),
                column![
                    text(i18n.t_with(
                        "categories-delete-question",
                        &args! { "name" => c.name.as_str() }
                    ))
                    .size(size::BODY)
                    .font(theme::SANS_SEMI),
                    super::muted(i18n.t_with(
                        "categories-delete-consequence",
                        &args! { "count" => u.count },
                    )),
                ]
                .spacing(2)
                .width(Length::Fill),
                small_button(i18n.t("action-cancel"))
                    .style(theme::secondary)
                    .on_press(Message::CancelDeleteCategory),
                small_button(i18n.t("action-delete"))
                    .style(theme::solid(Ink::Expense))
                    .on_press(Message::ConfirmDeleteCategory),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([10, 12])
        .style(theme::alert(Ink::Expense))
        .into();
    }

    let renaming = m.renaming.as_ref().filter(|(id, _)| *id == c.id);
    let name: Element<'a, Message> = match renaming {
        Some((_, value)) => text_input(&i18n.t("categories-rename-placeholder"), value)
            .id(Id::new(ids::RENAME))
            .on_input(Message::RenameText)
            .on_submit(Message::SaveRename)
            .padding([8, 12])
            .size(size::BODY)
            .style(theme::input)
            .into(),
        None => row![
            container(space().width(8).height(8)).style(theme::fill(accent, 4.0)),
            text(&c.name).size(size::BODY).font(theme::SANS_SEMI),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .into(),
    };

    let figure = |s: String, width: f32| {
        text(s)
            .size(size::BODY)
            .font(theme::NUM)
            .style(theme::ink(if u.count == 0 {
                Ink::Faint
            } else {
                Ink::Primary
            }))
            .width(width)
            .align_x(Horizontal::Right)
    };
    let actions: Element<'a, Message> = if renaming.is_some() {
        row![
            small_button(i18n.t("action-cancel"))
                .style(theme::secondary)
                .on_press(Message::CancelRename),
            small_button(i18n.t("action-save"))
                .style(theme::primary)
                .on_press(Message::SaveRename),
        ]
        .spacing(8)
        .into()
    } else {
        row![
            button(
                row![
                    icon(&icons::PENCIL, 16.0, Ink::Secondary),
                    text(i18n.t("action-rename")).size(size::SMALL)
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .padding([8, 10])
            .style(theme::ghost)
            .on_press(Message::StartRename(c.id)),
            button(container(icon(&icons::TRASH, 17.0, Ink::Secondary)).center(34))
                .padding(0)
                .style(theme::danger_ghost)
                .on_press(Message::AskDeleteCategory(c.id)),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    };

    let year_total = if u.year_total.is_zero() {
        "–".to_string()
    } else {
        i18n.money(u.year_total, account.currency)
    };

    container(
        row![
            container(name).width(Length::Fill),
            figure(u.count.to_string(), COUNT_WIDTH),
            figure(year_total, TOTAL_WIDTH),
            container(actions)
                .width(ACTIONS_WIDTH)
                .align_x(Horizontal::Right),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .height(52)
    .center_y(52)
    .style(theme::tinted(renaming.map(|_| Ink::Gold)))
    .into()
}
