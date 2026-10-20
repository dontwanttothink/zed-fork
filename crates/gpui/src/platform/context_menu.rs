use std::rc::Rc;

use thiserror::Error;

use crate::{
    Action, App, Bounds, Menu, MenuItem, OsAction, OsMenu, Pixels, Point, SharedString, Window,
    popup::PopupAnchor,
};

/// A context menu row's behavior, invoked with app and window access when the row is picked.
pub type ContextMenuItemHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Invokes a [`ContextMenuItemHandler`] in app context.
pub type ContextMenuItemHandlerRunner = Rc<dyn Fn(ContextMenuItemHandler)>;

/// An item in a platform-native context menu.
pub enum ContextMenuItem {
    /// A separator between items.
    Separator,

    /// Non-interactive text. When `header` is set, the platform may style it as a
    /// section header.
    Label {
        /// The text of the label.
        title: SharedString,
        /// Whether this label introduces a section of the menu.
        header: bool,
    },

    /// A pickable row.
    Item {
        /// The name shown for this item.
        name: SharedString,
        /// The row's identity: resolves the shortcut glyph, and is dispatched on
        /// pick when `handler` is `None`.
        action: Option<Box<dyn Action>>,
        /// The behavior to run on pick. Takes precedence over dispatching `action`.
        handler: Option<ContextMenuItemHandler>,
        /// The OS action corresponding to `action`, if any. Note that unlike menu
        /// bar rows, `os_action` rows here dispatch `action` app-side rather than
        /// through the OS responder chain.
        os_action: Option<OsAction>,
        /// Whether this item is checked.
        checked: bool,
        /// Whether this item can be chosen.
        disabled: bool,
    },

    /// A submenu. Items of any kind may appear inside.
    Submenu {
        /// The name of the submenu.
        name: SharedString,
        /// The submenu's items.
        items: Vec<ContextMenuItem>,
    },

    /// A menu, managed by the system (for example, the Services menu on macOS).
    SystemMenu(OsMenu),
}

impl Clone for ContextMenuItem {
    fn clone(&self) -> Self {
        match self {
            Self::Separator => Self::Separator,
            Self::Label { title, header } => Self::Label {
                title: title.clone(),
                header: *header,
            },
            Self::Item {
                name,
                action,
                handler,
                os_action,
                checked,
                disabled,
            } => Self::Item {
                name: name.clone(),
                action: action.as_ref().map(|action| action.boxed_clone()),
                handler: handler.clone(),
                os_action: *os_action,
                checked: *checked,
                disabled: *disabled,
            },
            Self::Submenu { name, items } => Self::Submenu {
                name: name.clone(),
                items: items.clone(),
            },
            Self::SystemMenu(os_menu) => Self::SystemMenu(os_menu.clone()),
        }
    }
}

/// Options for a platform-native context menu.
pub struct ContextMenuOptions {
    /// The rectangle the menu is positioned relative to, in the parent window's
    /// coordinate space (the same space element bounds are in). See
    /// [`crate::popup::PopupOptions::anchor_rect`].
    pub anchor_rect: Bounds<Pixels>,
    /// Which point of [`Self::anchor_rect`] the menu is anchored to.
    pub anchor: PopupAnchor,
    /// An additional offset applied to the menu after anchoring.
    pub offset: Point<Pixels>,
}

/// The result of presenting a context menu. Handlers for a picked item have run
/// by the time [`Self::Selected`] is delivered.
#[derive(Debug)]
pub enum ContextMenuOutcome {
    /// An item was chosen.
    Selected,
    /// Closed without choosing anything.
    Dismissed,
}

/// Returned when the current platform has no native context menu implementation.
#[derive(Debug, Error)]
#[error("native context menus are not supported on this platform")]
pub struct ContextMenuNotSupportedError;

impl From<MenuItem> for ContextMenuItem {
    fn from(value: MenuItem) -> Self {
        match value {
            MenuItem::Separator => Self::Separator,
            MenuItem::Submenu(Menu { name, items, .. }) => Self::Submenu {
                name,
                items: items.into_iter().map(Into::into).collect(),
            },
            MenuItem::SystemMenu(os_menu) => Self::SystemMenu(os_menu),
            MenuItem::Action {
                name,
                action,
                os_action,
                checked,
                disabled,
            } => Self::Item {
                name,
                action: Some(action),
                handler: None,
                os_action,
                checked,
                disabled,
            },
        }
    }
}
