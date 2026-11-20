//! Screens. Each function reads the state and returns elements; none of them
//! changes anything.

mod about;
mod account_settings;
mod backup;
mod categories;
mod entries;
mod picker;
mod preferences;
mod report;
mod unlock;
mod welcome;

use std::sync::LazyLock;

use dindim_core::{Kind, Money, YearMonth};
use dindim_i18n::{I18n, Sign};
use iced::widget::svg::Handle;
use iced::widget::{
    Button, Column, Row, button, column, container, row, scrollable, space, stack, text, tooltip,
};
use iced::{Alignment, Element, Length, Padding};

use crate::app::{App, Message, Picker, Screen, Tab};
use crate::icons::{self, icon};
use crate::session::{Notice, OpenAccount};
use crate::theme::{self, Ink, size};
use crate::widget::popover::Popover;

pub const RAIL_WIDTH: f32 = 228.0;

pub fn root(app: &App) -> Element<'_, Message> {
    let main: Element<'_, Message> = match (&app.screen, app.active()) {
        (Screen::Account(_), Some(account)) => {
            let body = match app.tab {
                Tab::Entries => entries::view(app, account),
                Tab::Report => report::view(app, account),
                Tab::Categories => categories::view(app, account),
                Tab::Settings => account_settings::view(app, account),
            };
            column![header(app, account), body].into()
        }
        (Screen::Unlock(id), _) => unlock::view(app, id),
        (Screen::Backup, _) => backup::view(app),
        (Screen::Preferences, _) => preferences::view(app),
        (Screen::About, _) => about::view(app),
        _ => welcome::view(app),
    };
    let page = container(main)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::page);
    let mut layers = stack![row![rail(app), page]];
    if let Some(toast) = &app.toast {
        layers = layers.push(
            container(
                container(
                    row![
                        icon(
                            if toast.ok {
                                &icons::CHECK_CIRCLE
                            } else {
                                &icons::ALERT
                            },
                            18.0,
                            if toast.ok { Ink::Gold } else { Ink::Expense },
                        ),
                        text(&toast.text)
                            .size(size::BODY)
                            .style(theme::ink(Ink::RailText)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                )
                .padding([12, 18])
                .max_width(560)
                .style(theme::toast),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding {
                left: RAIL_WIDTH,
                bottom: 28.0,
                ..Padding::ZERO
            })
            .align_x(Alignment::Center)
            .align_y(Alignment::End),
        );
    }
    layers.into()
}

// Sidebar

pub fn wordmark<'a>(mark_size: f32, text_size: f32, on_dark: bool) -> Row<'a, Message> {
    row![
        icons::coin(mark_size),
        text("dindim")
            .font(theme::DISPLAY_BOLD)
            .size(text_size)
            .style(theme::ink(if on_dark {
                Ink::RailText
            } else {
                Ink::Primary
            })),
    ]
    .spacing(mark_size * 0.32)
    .align_y(Alignment::Center)
}

/// A rounded square with the first letter of the account name.
pub fn avatar<'a>(name: &str, seed: &str, side: f32) -> Element<'a, Message> {
    let initial: String = name
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".into());
    let hash = seed.bytes().fold(2_166_136_261_u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16_777_619)
    });
    container(
        text(initial)
            .font(theme::DISPLAY_BOLD)
            .size(side * 0.5)
            .style(theme::ink(Ink::AvatarText)),
    )
    .center(side)
    .style(theme::fill(Ink::Avatar(hash as usize), side * 0.3))
    .into()
}

fn rail_glyph<'a>(handle: &LazyLock<Handle>) -> Element<'a, Message> {
    container(icon(handle, 16.0, Ink::RailText))
        .center(30)
        .style(|t| container::Style {
            background: Some(theme::palette(t).rail_active.into()),
            border: iced::Border {
                radius: 9.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn rail_button<'a>(
    content: impl Into<Element<'a, Message>>,
    active: bool,
    message: Message,
) -> Button<'a, Message> {
    button(content)
        .width(Length::Fill)
        .padding([7, 10])
        .style(theme::rail_item(active))
        .on_press(message)
}

fn rail_link<'a>(
    glyph: &LazyLock<Handle>,
    label: String,
    active: bool,
    message: Message,
) -> Button<'a, Message> {
    rail_button(
        row![rail_glyph(glyph), text(label).size(size::BODY)]
            .spacing(12)
            .align_y(Alignment::Center),
        active,
        message,
    )
}

fn rail(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let current = match &app.screen {
        Screen::Account(id) | Screen::Unlock(id) => Some(id.as_str()),
        _ => None,
    };

    let mut list = column![
        container(
            text(i18n.t("rail-accounts"))
                .size(size::CAPTION)
                .font(theme::SANS_SEMI)
                .style(theme::ink(Ink::RailMuted))
        )
        .padding([0, 10])
    ]
    .spacing(4);
    for info in &app.accounts {
        let is_open = app.open.iter().any(|a| a.info.id == info.id);
        let mut line = row![
            avatar(&info.name, &info.id, 30.0),
            text(&info.name)
                .size(size::BODY)
                .font(theme::SANS_SEMI)
                .wrapping(text::Wrapping::None),
            space::horizontal(),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        if info.protected && !is_open {
            line = line.push(icon(&icons::LOCK, 15.0, Ink::RailMuted));
        }
        list = list.push(rail_button(
            line,
            current == Some(info.id.as_str()),
            Message::SelectAccount(info.id.clone()),
        ));
    }
    list = list.push(rail_link(
        &icons::PLUS,
        i18n.t("rail-new-account"),
        app.screen == Screen::Welcome,
        Message::ShowWelcome,
    ));

    let mut footer = column![].spacing(4);
    if let Some(release) = &app.update {
        footer = footer.push(
            button(
                row![
                    icon(&icons::SPARKLE, 16.0, Ink::Gold),
                    text(i18n.t_with(
                        "rail-update-available",
                        &dindim_i18n::args! { "version" => release.version.to_string() },
                    ))
                    .size(size::SMALL)
                    .style(theme::ink(Ink::RailText)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding([8, 10])
            .width(Length::Fill)
            .style(theme::rail_item(false))
            .on_press(Message::OpenUrl(release.url.clone())),
        );
    }
    footer = footer
        .push(rail_link(
            &icons::ARCHIVE,
            i18n.t("rail-backup"),
            app.screen == Screen::Backup,
            Message::ShowBackup,
        ))
        .push(rail_link(
            &icons::SLIDERS,
            i18n.t("rail-preferences"),
            app.screen == Screen::Preferences,
            Message::ShowPreferences,
        ))
        .push(rail_link(
            &icons::INFO,
            i18n.t("rail-about"),
            app.screen == Screen::About,
            Message::ShowAbout,
        ));

    container(column![
        container(wordmark(32.0, 25.0, true)).padding([4, 10]),
        space().height(32),
        scrollable(list).height(Length::Fill).style(theme::scroll),
        space().height(12),
        footer,
    ])
    .padding([24, 14])
    .width(RAIL_WIDTH)
    .height(Length::Fill)
    .style(theme::rail)
    .into()
}

// Account header

const HEADER_ROW: f32 = 44.0;

fn header<'a>(app: &'a App, account: &'a OpenAccount) -> Element<'a, Message> {
    let i18n = &app.i18n;
    let mut top = row![
        text(&account.info.name)
            .font(theme::DISPLAY_BOLD)
            .size(size::HEADING)
            .wrapping(text::Wrapping::None),
        space::horizontal(),
    ]
    .spacing(12)
    .height(HEADER_ROW)
    .align_y(Alignment::Center);
    top = top
        .push(hint_button(
            &icons::UNDO,
            i18n.t("action-undo"),
            account.history.can_undo().then_some(Message::Undo),
        ))
        .push(hint_button(
            &icons::REDO,
            i18n.t("action-redo"),
            account.history.can_redo().then_some(Message::Redo),
        ));
    if matches!(app.tab, Tab::Entries | Tab::Report) {
        top = top.push(space().width(8)).push(month_nav(app));
    }

    let tab = |label: String, t: Tab| {
        let active = app.tab == t;
        let underline: Element<'a, Message> = if active {
            container(space().width(Length::Fill).height(3))
                .style(theme::fill(Ink::Gold, 2.0))
                .into()
        } else {
            space().height(3).into()
        };
        button(
            column![
                text(label).size(size::BODY).font(theme::SANS_SEMI),
                underline
            ]
            .spacing(10)
            .width(Length::Shrink),
        )
        .width(Length::Shrink)
        .padding(0)
        .style(theme::tab(active))
        .on_press(Message::SetTab(t))
    };

    column![
        top,
        space().height(12),
        row![
            tab(i18n.t("tab-entries"), Tab::Entries),
            tab(i18n.t("tab-report"), Tab::Report),
            tab(i18n.t("tab-categories"), Tab::Categories),
            tab(i18n.t("tab-settings"), Tab::Settings),
        ]
        .spacing(28),
        divider(),
    ]
    .padding(Padding {
        top: 20.0,
        right: 32.0,
        bottom: 0.0,
        left: 32.0,
    })
    .into()
}

fn month_nav(app: &App) -> Element<'_, Message> {
    let i18n = &app.i18n;
    let this_month = YearMonth::of(app.today);
    let open = app.picker == Some(Picker::Month);
    let label = button(
        container(
            row![
                text(capitalize(&i18n.month_year(app.month)))
                    .font(theme::DISPLAY)
                    .size(size::LEAD)
                    .wrapping(text::Wrapping::None),
                icon(&icons::CHEVRON_DOWN, 16.0, Ink::Secondary),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .center_x(Length::Fill),
    )
    .width(212)
    .padding([7, 10])
    .style(theme::month_button(open))
    .on_press(Message::TogglePicker(Picker::Month));
    let popup = open.then(|| picker::month_grid(i18n, app.picker_view, app.month, this_month));

    row![
        icon_button(&icons::CHEVRON_LEFT, Message::PrevMonth),
        Popover::new(label, popup, Message::ClosePicker),
        icon_button(&icons::CHEVRON_RIGHT, Message::NextMonth),
        space().width(8),
        button(
            text(i18n.t("action-this-month"))
                .size(size::SMALL)
                .font(theme::SANS_SEMI)
        )
        .padding([9, 14])
        .style(theme::secondary)
        .on_press_maybe((app.month != this_month).then_some(Message::GoToMonth(this_month))),
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .into()
}

// Shared pieces

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn divider<'a>() -> Element<'a, Message> {
    container(space().width(Length::Fill).height(1))
        .style(theme::fill(Ink::Line, 0.0))
        .into()
}

pub fn vrule<'a>(height: f32) -> Element<'a, Message> {
    container(space().width(1).height(height))
        .style(theme::fill(Ink::Line, 0.0))
        .into()
}

pub fn icon_button<'a>(handle: &LazyLock<Handle>, message: Message) -> Button<'a, Message> {
    button(container(icon(handle, 18.0, Ink::Secondary)).center(36))
        .padding(0)
        .style(theme::ghost)
        .on_press(message)
}

/// Icon button with a tooltip, disabled when `message` is `None`.
pub fn hint_button<'a>(
    handle: &LazyLock<Handle>,
    hint: String,
    message: Option<Message>,
) -> Element<'a, Message> {
    let ink = if message.is_some() {
        Ink::Secondary
    } else {
        Ink::Line
    };
    tooltip(
        button(container(icon(handle, 18.0, ink)).center(36))
            .padding(0)
            .style(theme::ghost)
            .on_press_maybe(message),
        container(
            text(hint)
                .size(size::SMALL)
                .style(theme::ink(Ink::RailText)),
        )
        .padding([6, 10])
        .style(theme::toast),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .into()
}

pub fn labeled<'a>(handle: &LazyLock<Handle>, label: String, ink: Ink) -> Row<'a, Message> {
    row![
        icon(handle, 17.0, ink),
        text(label).size(size::BODY).font(theme::SANS_SEMI)
    ]
    .spacing(8)
    .align_y(Alignment::Center)
}

/// Accent color of each entry kind.
pub fn kind_ink(kind: Kind) -> Ink {
    match kind {
        Kind::Expense => Ink::Expense,
        Kind::Income => Ink::Income,
        Kind::ReserveDeposit | Kind::ReserveWithdrawal => Ink::Reserve,
    }
}

pub fn kind_icon(kind: Kind) -> &'static LazyLock<Handle> {
    match kind {
        Kind::Expense => &icons::ARROW_OUT,
        Kind::Income => &icons::ARROW_IN,
        Kind::ReserveDeposit | Kind::ReserveWithdrawal => &icons::COINS,
    }
}

/// Amount with its sign from the wallet's point of view.
pub fn signed_amount(i18n: &I18n, account: &OpenAccount, kind: Kind, amount: Money) -> String {
    let sign = if kind.cash_effect(amount).is_negative() {
        Sign::Minus
    } else {
        Sign::Plus
    };
    i18n.money_signed(amount, account.currency, sign)
}

pub fn net_amount(i18n: &I18n, account: &OpenAccount, net: Money) -> String {
    let sign = if net.is_negative() {
        Sign::Minus
    } else {
        Sign::Plus
    };
    i18n.money_signed(net, account.currency, sign)
}

pub fn notice(n: &Option<Notice>) -> Option<Element<'_, Message>> {
    let n = n.as_ref()?;
    let (role, glyph) = if n.ok {
        (Ink::Income, &icons::CHECK_CIRCLE)
    } else {
        (Ink::Expense, &icons::ALERT)
    };
    Some(
        container(
            row![
                icon(glyph, 18.0, role),
                text(&n.text).size(size::BODY).width(Length::Fill)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([11, 14])
        .width(Length::Fill)
        .style(theme::alert(role))
        .into(),
    )
}

pub fn error_line<'a>(message: String) -> Element<'a, Message> {
    row![
        icon(&icons::ALERT, 16.0, Ink::Expense),
        text(message)
            .size(size::SMALL)
            .style(theme::ink(Ink::Expense))
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

pub fn label<'a>(content: String) -> iced::widget::Text<'a> {
    text(content)
        .size(size::SMALL)
        .font(theme::SANS_SEMI)
        .style(theme::ink(Ink::Secondary))
}

pub fn field<'a>(title: String, input: impl Into<Element<'a, Message>>) -> Column<'a, Message> {
    column![label(title), input.into()].spacing(7)
}

pub fn section_title<'a>(title: String) -> iced::widget::Text<'a> {
    text(title).font(theme::DISPLAY).size(size::TITLE)
}

pub fn muted<'a>(content: String) -> iced::widget::Text<'a> {
    text(content)
        .size(size::SMALL)
        .style(theme::ink(Ink::Secondary))
}

pub type Wrap<'a> = Box<dyn FnOnce(Button<'a, Message>) -> Element<'a, Message> + 'a>;

pub struct Segment<'a> {
    pub label: String,
    pub icon: Option<&'static LazyLock<Handle>>,
    pub active: bool,
    pub accent: Ink,
    pub message: Message,
    pub portion: u16,
    pub wrap: Option<Wrap<'a>>,
}

impl<'a> Segment<'a> {
    pub fn new(label: impl Into<String>, active: bool, accent: Ink, message: Message) -> Self {
        Segment {
            label: label.into(),
            icon: None,
            active,
            accent,
            message,
            portion: 1,
            wrap: None,
        }
    }

    pub fn icon(mut self, handle: &'static LazyLock<Handle>) -> Self {
        self.icon = Some(handle);
        self
    }

    pub fn portion(mut self, portion: u16) -> Self {
        self.portion = portion;
        self
    }

    /// Wraps the button, for example to attach a popover to it.
    pub fn wrap(
        mut self,
        f: impl FnOnce(Button<'a, Message>) -> Element<'a, Message> + 'a,
    ) -> Self {
        self.wrap = Some(Box::new(f));
        self
    }
}

pub fn segmented<'a>(segments: Vec<Segment<'a>>) -> Element<'a, Message> {
    let mut bar = Row::new().spacing(4);
    for s in segments {
        let ink = if s.active { s.accent } else { Ink::Secondary };
        let mut content = Row::new().spacing(6).align_y(Alignment::Center);
        if let Some(handle) = s.icon {
            content = content.push(icon(handle, 16.0, ink));
        }
        content = content.push(
            text(s.label)
                .size(size::SMALL + 1.0)
                .font(theme::SANS_SEMI)
                .wrapping(text::Wrapping::None),
        );
        let b = button(container(content).center(Length::Fill))
            .width(Length::Fill)
            .height(38)
            .padding([0, 6])
            .style(theme::segment(s.active, s.accent))
            .on_press(s.message);
        let element: Element<'a, Message> = match s.wrap {
            Some(wrap) => wrap(b),
            None => b.into(),
        };
        bar = bar.push(container(element).width(Length::FillPortion(s.portion)));
    }
    container(bar)
        .padding(4)
        .width(Length::Fill)
        .style(theme::sunk)
        .into()
}

pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> container::Container<'a, Message> {
    container(content)
        .padding(24)
        .width(Length::Fill)
        .style(theme::card)
}

/// A settings block: title and explanation on the left, controls on the right.
pub fn setting_row<'a>(
    title: String,
    description: String,
    controls: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        column![
            text(title).font(theme::DISPLAY).size(size::LEAD),
            muted(description),
        ]
        .spacing(6)
        .width(Length::FillPortion(2)),
        container(controls.into()).width(Length::FillPortion(3)),
    ]
    .spacing(40)
    .into()
}

/// A page that scrolls, with content centered up to `max_width`.
pub fn centered<'a>(
    content: impl Into<Element<'a, Message>>,
    max_width: f32,
) -> Element<'a, Message> {
    scrollable(
        container(container(content).max_width(max_width))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding([56, 32]),
    )
    .height(Length::Fill)
    .style(theme::scroll)
    .into()
}

pub fn wide_button<'a>(label: String) -> Button<'a, Message> {
    button(container(text(label).size(size::BODY).font(theme::SANS_BOLD)).center_x(Length::Fill))
        .width(Length::Fill)
        .padding([13, 18])
}

pub fn input<'a>(
    placeholder: &str,
    value: &str,
    on_input: fn(String) -> Message,
) -> iced::widget::TextInput<'a, Message> {
    iced::widget::text_input(placeholder, value)
        .on_input(on_input)
        .padding([12, 14])
        .size(size::BODY)
        .style(theme::input)
}

/// A labeled choice for pick lists whose labels come from translations.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice<T> {
    pub value: T,
    pub label: String,
}

impl<T> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
