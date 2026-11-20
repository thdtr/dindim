//! Contents of the floating panels: a calendar and a month picker.

use dindim_core::{Date, YearMonth};
use dindim_i18n::I18n;
use iced::alignment::Horizontal;
use iced::widget::{Column, Row, button, column, container, row, text};
use iced::{Alignment, Element, Length};

use super::{capitalize, icon_button};
use crate::app::Message;
use crate::icons;
use crate::theme::{self, Ink, size};

const CELL: f32 = 38.0;
const MONTH_WIDTH: f32 = 82.0;

fn header<'a>(title: String, prev: Message, next: Message) -> Element<'a, Message> {
    row![
        icon_button(&icons::CHEVRON_LEFT, prev),
        text(title)
            .font(theme::DISPLAY)
            .size(size::BODY + 1.0)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        icon_button(&icons::CHEVRON_RIGHT, next),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// A month of days. `view` is the month shown, `selected` is drawn in gold
/// and `today` gets a ring. Weeks start on Sunday.
pub fn calendar<'a>(
    i18n: &I18n,
    view: YearMonth,
    selected: Date,
    today: Date,
    on_pick: impl Fn(Date) -> Message,
    footer: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let first = view.first_day();
    let offset = i64::from(first.weekday().to_sunday_zero_offset());
    let start = first
        .checked_sub(jiff::Span::new().days(offset))
        .unwrap_or(first);

    let mut weeks = Column::new().spacing(2);
    weeks = weeks.push(Row::with_children(i18n.weekday_initials().into_iter().map(
        |d| {
            text(d)
                .size(12)
                .font(theme::SANS_SEMI)
                .style(theme::ink(Ink::Faint))
                .width(CELL)
                .align_x(Horizontal::Center)
                .into()
        },
    )));
    for week in 0..6 {
        let mut line = Row::new();
        for weekday in 0..7 {
            let Ok(day) = start.checked_add(jiff::Span::new().days(week * 7 + weekday)) else {
                continue;
            };
            let is_selected = day == selected;
            line =
                line.push(
                    button(
                        container(text(day.day().to_string()).size(size::SMALL).font(
                            if is_selected {
                                theme::SANS_BOLD
                            } else {
                                theme::SANS
                            },
                        ))
                        .center(CELL),
                    )
                    .padding(0)
                    .style(theme::day(is_selected, day == today, !view.contains(day)))
                    .on_press(on_pick(day)),
                );
        }
        weeks = weeks.push(line);
    }

    let mut body = column![
        header(
            capitalize(&i18n.month_year(view)),
            Message::PickerView(view.prev()),
            Message::PickerView(view.next()),
        ),
        weeks,
    ]
    .spacing(10);
    if let Some(footer) = footer {
        body = body.push(footer);
    }
    container(body)
        .padding(14)
        .width(7.0 * CELL + 28.0)
        .style(theme::popover)
        .into()
}

/// The twelve months of a year in a grid.
pub fn month_grid<'a>(
    i18n: &I18n,
    view: YearMonth,
    current: YearMonth,
    this_month: YearMonth,
) -> Element<'a, Message> {
    let year = view.year();
    let mut grid = Column::new().spacing(6);
    for line in 0..4 {
        let mut r = Row::new().spacing(6);
        for col in 1..=3 {
            let Some(month) = YearMonth::new(year, line * 3 + col) else {
                continue;
            };
            r = r.push(
                button(
                    container(
                        text(capitalize(&i18n.month_short(month.month())))
                            .size(size::SMALL + 1.0)
                            .font(theme::SANS_SEMI),
                    )
                    .center_x(Length::Fill)
                    .center_y(36),
                )
                .width(MONTH_WIDTH)
                .padding(0)
                .style(theme::day(month == current, month == this_month, false))
                .on_press(Message::GoToMonth(month)),
            );
        }
        grid = grid.push(r);
    }
    let shift = |delta: i16| {
        Message::PickerView(
            year.checked_add(delta)
                .and_then(|y| view.with_year(y))
                .unwrap_or(view),
        )
    };
    container(column![header(year.to_string(), shift(-1), shift(1)), grid].spacing(10))
        .padding(14)
        .width(3.0 * MONTH_WIDTH + 2.0 * 6.0 + 28.0)
        .style(theme::popover)
        .into()
}
