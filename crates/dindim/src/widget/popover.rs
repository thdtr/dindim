//! A floating panel attached to an element, used for calendars and menus.
//!
//! The panel opens right below the anchor (or above it when there is no
//! room), on top of everything else. A click outside it produces `on_dismiss`;
//! clicks on the anchor itself are left to the anchor, which is how a button
//! can both open and close its panel.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse, touch};
use iced::{Renderer, Theme};

const EDGE: f32 = 8.0;

pub struct Popover<'a, Message> {
    anchor: Element<'a, Message>,
    popup: Option<Element<'a, Message>>,
    on_dismiss: Message,
    gap: f32,
    align_end: bool,
    match_width: bool,
}

impl<Message> std::fmt::Debug for Popover<'_, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Popover")
            .field("open", &self.popup.is_some())
            .finish_non_exhaustive()
    }
}

impl<'a, Message> Popover<'a, Message> {
    /// `popup` is `None` while the panel is closed.
    pub fn new(
        anchor: impl Into<Element<'a, Message>>,
        popup: Option<Element<'a, Message>>,
        on_dismiss: Message,
    ) -> Self {
        Popover {
            anchor: anchor.into(),
            popup,
            on_dismiss,
            gap: 6.0,
            align_end: false,
            match_width: false,
        }
    }

    /// Lines up the right edge of the panel with the anchor.
    pub fn align_end(mut self) -> Self {
        self.align_end = true;
        self
    }

    /// Makes the panel exactly as wide as the anchor.
    pub fn match_width(mut self) -> Self {
        self.match_width = true;
        self
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Popover<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        let mut trees = vec![Tree::new(&self.anchor)];
        if let Some(popup) = &self.popup {
            trees.push(Tree::new(popup));
        }
        trees
    }

    fn diff(&self, tree: &mut Tree) {
        let mut children: Vec<&dyn Widget<Message, Theme, Renderer>> =
            vec![self.anchor.as_widget()];
        if let Some(popup) = &self.popup {
            children.push(popup.as_widget());
        }
        tree.diff_children(&children);
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let (anchor_tree, rest) = tree.children.split_first_mut()?;
        match (&mut self.popup, rest.first_mut()) {
            (Some(popup), Some(popup_tree)) => Some(overlay::Element::new(Box::new(Panel {
                popup,
                tree: popup_tree,
                anchor: layout.bounds() + translation,
                gap: self.gap,
                align_end: self.align_end,
                match_width: self.match_width,
                on_dismiss: self.on_dismiss.clone(),
            }))),
            _ => self.anchor.as_widget_mut().overlay(
                anchor_tree,
                layout,
                renderer,
                viewport,
                translation,
            ),
        }
    }
}

impl<'a, Message: Clone + 'a> From<Popover<'a, Message>> for Element<'a, Message> {
    fn from(popover: Popover<'a, Message>) -> Self {
        Element::new(popover)
    }
}

struct Panel<'a, 'b, Message> {
    popup: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    anchor: Rectangle,
    gap: f32,
    align_end: bool,
    match_width: bool,
    on_dismiss: Message,
}

impl<Message: Clone> overlay::Overlay<Message, Theme, Renderer> for Panel<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = if self.match_width {
            layout::Limits::new(
                Size::new(self.anchor.width, 0.0),
                Size::new(self.anchor.width, bounds.height),
            )
        } else {
            layout::Limits::new(Size::ZERO, bounds)
        };
        let node = self
            .popup
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();

        let x = if self.align_end {
            self.anchor.x + self.anchor.width - size.width
        } else {
            self.anchor.x
        };
        let x = x.clamp(EDGE, (bounds.width - size.width - EDGE).max(EDGE));

        let below = self.anchor.y + self.anchor.height + self.gap;
        let above = self.anchor.y - self.gap - size.height;
        let y = if below + size.height > bounds.height - EDGE && above >= EDGE {
            above
        } else {
            below
        };
        node.move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.popup.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.popup
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let pressed = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(_))
                | Event::Touch(touch::Event::FingerPressed { .. })
        );
        self.popup.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
        if pressed {
            let outside = cursor
                .position()
                .is_some_and(|p| !layout.bounds().contains(p) && !self.anchor.contains(p));
            if outside {
                shell.publish(self.on_dismiss.clone());
            }
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.popup.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}
