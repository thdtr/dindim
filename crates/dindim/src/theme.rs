//! Design system: gold coin on ink, in a light and a dark palette.
//!
//! Red, green and blue only appear for what they mean (money out, money in,
//! emergency fund), never as decoration. Every text color keeps a contrast of
//! at least 4.5:1 against the surfaces it sits on.

use iced::border::Radius;
use iced::font::Weight;
use iced::widget::{button, container, pick_list, scrollable, svg, text, text_editor, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector, color};

// Typography

const FIGTREE: Font = Font::with_name("Figtree");
pub const SANS: Font = Font {
    weight: Weight::Medium,
    ..FIGTREE
};
pub const SANS_SEMI: Font = Font {
    weight: Weight::Semibold,
    ..FIGTREE
};
pub const SANS_BOLD: Font = Font {
    weight: Weight::Bold,
    ..FIGTREE
};
/// Tabular figures, so amounts line up in lists and fields.
pub const NUM: Font = Font {
    weight: Weight::Semibold,
    ..Font::with_name("Figtree Tabular")
};
pub const NUM_BOLD: Font = Font {
    weight: Weight::Bold,
    ..Font::with_name("Figtree Tabular")
};
pub const DISPLAY: Font = Font {
    weight: Weight::Semibold,
    ..Font::with_name("Bricolage Grotesque")
};
pub const DISPLAY_BOLD: Font = Font {
    weight: Weight::ExtraBold,
    ..Font::with_name("Bricolage Grotesque")
};

pub mod size {
    pub const CAPTION: f32 = 13.0;
    pub const SMALL: f32 = 14.0;
    pub const BODY: f32 = 16.0;
    pub const LEAD: f32 = 18.0;
    pub const TITLE: f32 = 22.0;
    pub const HEADING: f32 = 28.0;
    pub const HERO: f32 = 40.0;
}

pub const R_SM: f32 = 8.0;
pub const R_MD: f32 = 12.0;
pub const R_LG: f32 = 16.0;
pub const PILL: f32 = 999.0;

// Palettes

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub page: Color,
    pub card: Color,
    pub sunk: Color,
    pub hover: Color,
    pub line: Color,
    pub line_strong: Color,
    pub ink: Color,
    pub ink_2: Color,
    pub ink_3: Color,
    pub gold: Color,
    pub gold_hover: Color,
    pub gold_pressed: Color,
    pub gold_tint: Color,
    pub gold_text: Color,
    pub on_gold: Color,
    pub rail: Color,
    pub rail_hover: Color,
    pub rail_active: Color,
    pub rail_text: Color,
    pub rail_muted: Color,
    pub expense: Color,
    pub expense_tint: Color,
    pub income: Color,
    pub income_tint: Color,
    pub reserve: Color,
    pub reserve_tint: Color,
    pub on_accent: Color,
    pub shadow: Color,
}

pub const LIGHT: Palette = Palette {
    page: color!(0xF5F3EE),
    card: color!(0xFFFFFF),
    sunk: color!(0xECE9E2),
    hover: color!(0xF3F0EA),
    line: color!(0xE3DFD6),
    line_strong: color!(0xCDC7BB),
    ink: color!(0x1C1B19),
    ink_2: color!(0x56524A),
    ink_3: color!(0x7A756B),
    gold: color!(0xF2B300),
    gold_hover: color!(0xFFC52E),
    gold_pressed: color!(0xDDA300),
    gold_tint: color!(0xFCEFC7),
    gold_text: color!(0x8A6000),
    on_gold: color!(0x1C1B19),
    rail: color!(0x1C1B19),
    rail_hover: color!(0x292824),
    rail_active: color!(0x34322D),
    rail_text: color!(0xF1EEE7),
    rail_muted: color!(0xA39E93),
    expense: color!(0xB4321F),
    expense_tint: color!(0xFBE6E2),
    income: color!(0x18704A),
    income_tint: color!(0xDFF1E6),
    reserve: color!(0x2D5BA6),
    reserve_tint: color!(0xE2EAF7),
    on_accent: color!(0xFFFFFF),
    shadow: color!(0x1C1B19),
};

pub const DARK: Palette = Palette {
    page: color!(0x161513),
    card: color!(0x201F1C),
    sunk: color!(0x2A2824),
    hover: color!(0x272521),
    line: color!(0x35322D),
    line_strong: color!(0x4B473F),
    ink: color!(0xF1EEE7),
    ink_2: color!(0xBDB7AB),
    ink_3: color!(0x908A7F),
    gold: color!(0xF2B300),
    gold_hover: color!(0xFFC52E),
    gold_pressed: color!(0xDDA300),
    gold_tint: color!(0x3B3116),
    gold_text: color!(0xF5C842),
    on_gold: color!(0x1C1B19),
    rail: color!(0x0F0E0D),
    rail_hover: color!(0x1C1B18),
    rail_active: color!(0x2A2824),
    rail_text: color!(0xF1EEE7),
    rail_muted: color!(0x9C978C),
    expense: color!(0xF28A74),
    expense_tint: color!(0x3B201A),
    income: color!(0x5CCB94),
    income_tint: color!(0x16301F),
    reserve: color!(0x86AEF0),
    reserve_tint: color!(0x1B2840),
    on_accent: color!(0x161513),
    shadow: color!(0x000000),
};

pub const AVATARS: [Color; 6] = [
    color!(0xF2B300),
    color!(0xF4A28C),
    color!(0x8FCFAD),
    color!(0x9DBEEB),
    color!(0xC9B2EC),
    color!(0xE6D3A8),
];
pub const AVATAR_TEXT: Color = color!(0x1C1B19);

pub fn theme(dark: bool) -> Theme {
    let p = if dark { DARK } else { LIGHT };
    Theme::custom(
        if dark { "dindim dark" } else { "dindim light" },
        iced::theme::Palette {
            background: p.page,
            text: p.ink,
            primary: p.gold,
            success: p.income,
            warning: p.gold,
            danger: p.expense,
        },
    )
}

pub fn palette(theme: &Theme) -> &'static Palette {
    if theme.extended_palette().is_dark {
        &DARK
    } else {
        &LIGHT
    }
}

/// A color by role, resolved against the current palette when drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Primary,
    Secondary,
    Faint,
    Line,
    LineStrong,
    Gold,
    GoldText,
    OnGold,
    Expense,
    Income,
    Reserve,
    RailText,
    RailMuted,
    Avatar(usize),
    AvatarText,
}

impl Ink {
    pub fn resolve(self, p: &Palette) -> Color {
        match self {
            Ink::Primary => p.ink,
            Ink::Secondary => p.ink_2,
            Ink::Faint => p.ink_3,
            Ink::Line => p.line,
            Ink::LineStrong => p.line_strong,
            Ink::Gold => p.gold,
            Ink::GoldText => p.gold_text,
            Ink::OnGold => p.on_gold,
            Ink::Expense => p.expense,
            Ink::Income => p.income,
            Ink::Reserve => p.reserve,
            Ink::RailText => p.rail_text,
            Ink::RailMuted => p.rail_muted,
            Ink::Avatar(i) => AVATARS[i % AVATARS.len()],
            Ink::AvatarText => AVATAR_TEXT,
        }
    }

    /// The light background that goes with an accent.
    pub fn tint(self, p: &Palette) -> Color {
        match self {
            Ink::Expense => p.expense_tint,
            Ink::Income => p.income_tint,
            Ink::Reserve => p.reserve_tint,
            Ink::Gold | Ink::GoldText => p.gold_tint,
            _ => p.sunk,
        }
    }
}

fn border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: Radius::from(radius),
    }
}

fn soft_shadow(p: &Palette, alpha: f32, y: f32, blur: f32) -> Shadow {
    Shadow {
        color: with_alpha(p.shadow, alpha),
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a,
    }
}

// Text and icons

pub fn ink(role: Ink) -> impl Fn(&Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(role.resolve(palette(theme))),
    }
}

pub fn icon_ink(role: Ink) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |theme, _| svg::Style {
        color: Some(role.resolve(palette(theme))),
    }
}

// Surfaces

pub fn page(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.page.into()),
        text_color: Some(p.ink),
        ..Default::default()
    }
}

pub fn rail(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.rail.into()),
        text_color: Some(p.rail_text),
        ..Default::default()
    }
}

pub fn card(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.card.into()),
        border: border(p.line, 1.0, R_LG),
        shadow: soft_shadow(p, 0.04, 1.0, 3.0),
        text_color: Some(p.ink),
        ..Default::default()
    }
}

pub fn popover(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.card.into()),
        border: border(p.line, 1.0, R_LG),
        shadow: soft_shadow(p, 0.18, 10.0, 28.0),
        text_color: Some(p.ink),
        ..Default::default()
    }
}

pub fn toast(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.rail.into()),
        border: border(Color::TRANSPARENT, 0.0, R_MD),
        shadow: soft_shadow(p, 0.25, 8.0, 24.0),
        text_color: Some(p.rail_text),
        ..Default::default()
    }
}

pub fn sunk(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette(theme).sunk.into()),
        border: border(Color::TRANSPARENT, 0.0, R_MD),
        ..Default::default()
    }
}

/// Background in the tint of an accent; `None` for transparent.
pub fn tinted(role: Option<Ink>) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        container::Style {
            background: role.map(|r| r.tint(p).into()),
            border: border(Color::TRANSPARENT, 0.0, R_MD),
            ..Default::default()
        }
    }
}

pub fn fill(role: Ink, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| container::Style {
        background: Some(role.resolve(palette(theme)).into()),
        border: border(Color::TRANSPARENT, 0.0, radius),
        ..Default::default()
    }
}

pub fn fill_tint(role: Ink, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| container::Style {
        background: Some(role.tint(palette(theme)).into()),
        border: border(Color::TRANSPARENT, 0.0, radius),
        ..Default::default()
    }
}

pub fn fill_sunk(radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| container::Style {
        background: Some(palette(theme).sunk.into()),
        border: border(Color::TRANSPARENT, 0.0, radius),
        ..Default::default()
    }
}

pub fn alert(role: Ink) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        container::Style {
            background: Some(role.tint(p).into()),
            border: border(with_alpha(role.resolve(p), 0.35), 1.0, R_MD),
            text_color: Some(p.ink),
            ..Default::default()
        }
    }
}

// Buttons

fn plain(bg: Option<Color>, fg: Color, edge: Border) -> button::Style {
    button::Style {
        background: bg.map(Background::Color),
        text_color: fg,
        border: edge,
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let bg = match status {
        button::Status::Hovered => p.gold_hover,
        button::Status::Pressed => p.gold_pressed,
        button::Status::Disabled => p.sunk,
        button::Status::Active => p.gold,
    };
    let fg = if status == button::Status::Disabled {
        p.ink_3
    } else {
        p.on_gold
    };
    plain(Some(bg), fg, border(Color::TRANSPARENT, 0.0, R_MD))
}

pub fn solid(role: Ink) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let accent = role.resolve(p);
        let bg = match status {
            button::Status::Hovered => mix(accent, Color::WHITE, 0.12),
            button::Status::Pressed => mix(accent, Color::BLACK, 0.15),
            button::Status::Disabled => p.sunk,
            button::Status::Active => accent,
        };
        let fg = if status == button::Status::Disabled {
            p.ink_3
        } else {
            p.on_accent
        };
        let mut style = plain(Some(bg), fg, border(Color::TRANSPARENT, 0.0, R_MD));
        if matches!(status, button::Status::Active | button::Status::Hovered) {
            style.shadow = Shadow {
                color: with_alpha(accent, 0.25),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            };
        }
        style
    }
}

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, edge, fg) = match status {
        button::Status::Hovered => (p.hover, p.line_strong, p.ink),
        button::Status::Pressed => (p.sunk, p.line_strong, p.ink),
        button::Status::Disabled => (p.card, p.line, p.ink_3),
        button::Status::Active => (p.card, p.line, p.ink),
    };
    plain(Some(bg), fg, border(edge, 1.0, R_MD))
}

pub fn ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let bg = match status {
        button::Status::Hovered => Some(p.hover),
        button::Status::Pressed => Some(p.sunk),
        _ => None,
    };
    let fg = if status == button::Status::Disabled {
        p.ink_3
    } else {
        p.ink_2
    };
    plain(bg, fg, border(Color::TRANSPARENT, 0.0, R_SM))
}

pub fn link(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let fg = match status {
        button::Status::Hovered | button::Status::Pressed => p.ink,
        _ => p.gold_text,
    };
    plain(None, fg, border(Color::TRANSPARENT, 0.0, 0.0))
}

pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, fg) = match status {
        button::Status::Hovered | button::Status::Pressed => (Some(p.expense_tint), p.expense),
        button::Status::Disabled => (None, p.ink_3),
        button::Status::Active => (Some(p.card), p.expense),
    };
    plain(bg, fg, border(with_alpha(p.expense, 0.5), 1.0, R_MD))
}

pub fn danger_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, fg) = match status {
        button::Status::Hovered | button::Status::Pressed => (Some(p.expense_tint), p.expense),
        _ => (None, p.ink_2),
    };
    plain(bg, fg, border(Color::TRANSPARENT, 0.0, R_SM))
}

/// Item of a segmented control. The active one becomes a card on the track.
pub fn segment(active: bool, role: Ink) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let edge = border(Color::TRANSPARENT, 0.0, 9.0);
        let mut style = if active {
            plain(Some(p.card), role.resolve(p), edge)
        } else if hovered {
            plain(Some(with_alpha(p.card, 0.55)), p.ink, edge)
        } else {
            plain(None, p.ink_2, edge)
        };
        if active {
            style.shadow = soft_shadow(p, 0.10, 1.0, 3.0);
        }
        style
    }
}

pub fn tab(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let fg = if active || matches!(status, button::Status::Hovered) {
            p.ink
        } else {
            p.ink_2
        };
        plain(None, fg, border(Color::TRANSPARENT, 0.0, 0.0))
    }
}

pub fn rail_item(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let bg = if active {
            Some(p.rail_active)
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(p.rail_hover)
        } else {
            None
        };
        plain(bg, p.rail_text, border(Color::TRANSPARENT, 0.0, R_MD))
    }
}

pub fn list_row(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (bg, edge) = if selected {
            (Some(p.gold_tint), with_alpha(p.gold, 0.8))
        } else if hovered {
            (Some(p.hover), Color::TRANSPARENT)
        } else {
            (None, Color::TRANSPARENT)
        };
        plain(bg, p.ink, border(edge, 1.0, R_MD))
    }
}

pub fn menu_item(highlighted: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if highlighted {
            Some(p.gold_tint)
        } else if hovered {
            Some(p.hover)
        } else {
            None
        };
        plain(bg, p.ink, border(Color::TRANSPARENT, 0.0, R_SM))
    }
}

/// Calendar day or month in the month picker.
pub fn day(
    selected: bool,
    today: bool,
    muted: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let fg = if selected {
            p.on_gold
        } else if muted {
            p.ink_3
        } else {
            p.ink
        };
        let bg = if selected {
            Some(p.gold)
        } else if hovered {
            Some(p.hover)
        } else {
            None
        };
        let edge = if today && !selected {
            border(p.gold_text, 1.5, PILL)
        } else {
            border(Color::TRANSPARENT, 0.0, PILL)
        };
        plain(bg, fg, edge)
    }
}

pub fn month_button(open: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let bg = if open {
            Some(p.sunk)
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(p.hover)
        } else {
            None
        };
        plain(bg, p.ink, border(Color::TRANSPARENT, 0.0, R_SM))
    }
}

// Fields

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (bg, edge, width) = match status {
        text_input::Status::Focused { .. } => (p.card, p.ink, 1.5),
        text_input::Status::Hovered => (p.card, p.line_strong, 1.0),
        text_input::Status::Disabled => (p.sunk, p.line, 1.0),
        text_input::Status::Active => (p.card, p.line, 1.0),
    };
    text_input::Style {
        background: Background::Color(bg),
        border: border(edge, width, R_MD),
        icon: p.ink_2,
        placeholder: p.ink_3,
        value: p.ink,
        selection: with_alpha(p.gold, 0.35),
    }
}

pub fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let status = match status {
        text_editor::Status::Active => text_input::Status::Active,
        text_editor::Status::Hovered => text_input::Status::Hovered,
        text_editor::Status::Focused { is_hovered } => text_input::Status::Focused { is_hovered },
        text_editor::Status::Disabled => text_input::Status::Disabled,
    };
    let s = input(theme, status);
    text_editor::Style {
        background: s.background,
        border: s.border,
        placeholder: s.placeholder,
        value: s.value,
        selection: s.selection,
    }
}

pub fn pick(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let p = palette(theme);
    let edge = match status {
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => p.line_strong,
        pick_list::Status::Active => p.line,
    };
    pick_list::Style {
        text_color: p.ink,
        placeholder_color: p.ink_3,
        handle_color: p.ink_2,
        background: Background::Color(p.card),
        border: border(edge, 1.0, R_MD),
    }
}

pub fn pick_menu(theme: &Theme) -> iced::overlay::menu::Style {
    let p = palette(theme);
    iced::overlay::menu::Style {
        background: p.card.into(),
        border: border(p.line_strong, 1.0, R_MD),
        text_color: p.ink,
        selected_text_color: p.ink,
        selected_background: p.gold_tint.into(),
        shadow: soft_shadow(p, 0.18, 8.0, 20.0),
    }
}

pub fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = palette(theme);
    let mut style = scrollable::default(theme, status);
    let hovered = matches!(
        status,
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | scrollable::Status::Dragged { .. }
    );
    let scroller = scrollable::Scroller {
        background: Background::Color(if hovered { p.line_strong } else { p.line }),
        border: border(Color::TRANSPARENT, 0.0, PILL),
    };
    style.vertical_rail = scrollable::Rail {
        background: None,
        border: border(Color::TRANSPARENT, 0.0, 0.0),
        scroller,
    };
    style
}
