//! Line icons (24×24, 2px stroke) and the brand mark. Icons are drawn in black
//! and colored by the style when rendered.

use std::sync::LazyLock;

use iced::Length;
use iced::widget::svg::{self, Handle};

use crate::theme::{self, Ink};

macro_rules! icon {
    ($name:ident, $body:literal) => {
        pub static $name: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(
                concat!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">"#,
                    $body,
                    "</svg>"
                )
                .as_bytes(),
            )
        });
    };
}

icon!(CHEVRON_LEFT, r#"<path d="M15 18l-6-6 6-6"/>"#);
icon!(CHEVRON_RIGHT, r#"<path d="M9 18l6-6-6-6"/>"#);
icon!(CHEVRON_DOWN, r#"<path d="M6 9l6 6 6-6"/>"#);
icon!(
    CALENDAR,
    r#"<rect x="3.5" y="5" width="17" height="15.5" rx="2.5"/><path d="M8 3v4M16 3v4M3.5 10h17"/>"#
);
icon!(PLUS, r#"<path d="M12 5v14M5 12h14"/>"#);
icon!(MINUS, r#"<path d="M5 12h14"/>"#);
icon!(
    PENCIL,
    r#"<path d="M4 20h4L19.5 8.5a2.1 2.1 0 0 0-3-3L5 17v3z"/><path d="M14.5 7.5l2 2"/>"#
);
icon!(
    TRASH,
    r#"<path d="M4 7h16M10 11v6M14 11v6M6 7l1 12.5a1.5 1.5 0 0 0 1.5 1.5h7a1.5 1.5 0 0 0 1.5-1.5L18 7M9 7V4.5A1.5 1.5 0 0 1 10.5 3h3A1.5 1.5 0 0 1 15 4.5V7"/>"#
);
icon!(
    SEARCH,
    r#"<circle cx="11" cy="11" r="6.5"/><path d="M20 20l-4.2-4.2"/>"#
);
icon!(
    LOCK,
    r#"<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7.5a4 4 0 0 1 8 0V11"/>"#
);
icon!(X, r#"<path d="M6 6l12 12M18 6L6 18"/>"#);
icon!(ARROW_OUT, r#"<path d="M7 17L17 7M8.5 7H17v8.5"/>"#);
icon!(ARROW_IN, r#"<path d="M17 7L7 17M15.5 17H7V8.5"/>"#);
icon!(
    COINS,
    r#"<ellipse cx="12" cy="6.5" rx="7.5" ry="3"/><path d="M4.5 6.5v5c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3v-5"/><path d="M4.5 11.5v5c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3v-5"/>"#
);
icon!(
    DOWNLOAD,
    r#"<path d="M12 4v11M7.5 10.5L12 15l4.5-4.5M5 20h14"/>"#
);
icon!(
    UPLOAD,
    r#"<path d="M12 15V4M7.5 8.5L12 4l4.5 4.5M5 20h14"/>"#
);
icon!(
    LOG_OUT,
    r#"<path d="M10 4H6.5A2.5 2.5 0 0 0 4 6.5v11A2.5 2.5 0 0 0 6.5 20H10M15 16l4-4-4-4M19 12H9.5"/>"#
);
icon!(
    ARCHIVE,
    r#"<rect x="3" y="4" width="18" height="5" rx="1.5"/><path d="M5 9v9.5A1.5 1.5 0 0 0 6.5 20h11a1.5 1.5 0 0 0 1.5-1.5V9M10 13h4"/>"#
);
icon!(
    ALERT,
    r#"<circle cx="12" cy="12" r="9"/><path d="M12 7.5v5.5M12 16.5v.01"/>"#
);
icon!(
    CHECK_CIRCLE,
    r#"<circle cx="12" cy="12" r="9"/><path d="M8 12.3l2.8 2.7L16 9.5"/>"#
);
icon!(
    TAG,
    r#"<path d="M3.5 12.1V4.5a1 1 0 0 1 1-1h7.6a1 1 0 0 1 .7.3l8 8a1 1 0 0 1 0 1.4l-7.6 7.6a1 1 0 0 1-1.4 0l-8-8a1 1 0 0 1-.3-.7z"/><circle cx="8" cy="8" r="1.2"/>"#
);

icon!(
    UNDO,
    r#"<path d="M9 14L4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/>"#
);
icon!(
    REDO,
    r#"<path d="M15 14l5-5-5-5"/><path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13"/>"#
);
icon!(
    SLIDERS,
    r#"<path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="18" r="2"/>"#
);
icon!(
    INFO,
    r#"<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5M12 7.5v.01"/>"#
);
icon!(
    FILE,
    r#"<path d="M14 3H7.5A2.5 2.5 0 0 0 5 5.5v13A2.5 2.5 0 0 0 7.5 21h9a2.5 2.5 0 0 0 2.5-2.5V8z"/><path d="M14 3v5h5M9 13h6M9 17h4"/>"#
);
icon!(
    EXTERNAL,
    r#"<path d="M14 4h6v6M20 4l-9 9"/><path d="M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"/>"#
);
icon!(
    SPARKLE,
    r#"<path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z"/><path d="M19 17v4M17 19h4"/>"#
);

pub static COIN: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(&include_bytes!("../../../assets/coin.svg")[..]));

pub static MARK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(&include_bytes!("../../../assets/coin-mono.svg")[..]));

pub fn icon<'a>(handle: &LazyLock<Handle>, size: f32, ink: Ink) -> svg::Svg<'a> {
    svg::Svg::new((**handle).clone())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(theme::icon_ink(ink))
}

/// The full color brand mark.
pub fn coin<'a>(size: f32) -> svg::Svg<'a> {
    svg::Svg::new(COIN.clone())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

/// The single color brand mark.
pub fn mark<'a>(size: f32, ink: Ink) -> svg::Svg<'a> {
    svg::Svg::new(MARK.clone())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(theme::icon_ink(ink))
}
