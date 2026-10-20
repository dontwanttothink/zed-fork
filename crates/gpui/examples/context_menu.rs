#![cfg_attr(target_family = "wasm", no_main)]

#[path = "example_support/fonts.rs"]
mod example_support;

use std::rc::Rc;

use gpui::{
    App, Bounds, Context, ContextMenuItem, ContextMenuItemHandler, ContextMenuOptions,
    ContextMenuOutcome, KeyBinding, MouseButton, MouseDownEvent, Point, SharedString, Size, Window,
    WindowBounds, WindowOptions, actions, div, popup::PopupAnchor, prelude::*, px, size,
};
use gpui_platform::application;

actions!(context_menu, [Rename, ToggleHiddenFiles, Trash]);

struct ContextMenuExample {
    show_hidden_files: bool,
    status: SharedString,
}

impl ContextMenuExample {
    fn handler(
        message: &'static str,
        this: gpui::WeakEntity<Self>,
        toggles_hidden_files: bool,
    ) -> ContextMenuItemHandler {
        Rc::new(move |_window, cx| {
            if let Err(error) = this.update(cx, |this, cx| {
                if toggles_hidden_files {
                    this.show_hidden_files = !this.show_hidden_files;
                }
                this.status = message.into();
                cx.notify();
            }) {
                eprintln!("could not update the example after a menu pick: {error}");
            }
        })
    }

    fn menu_items(&self, cx: &mut Context<Self>) -> Vec<ContextMenuItem> {
        let item = |name: &'static str,
                    action: Option<Box<dyn gpui::Action>>,
                    handler: ContextMenuItemHandler| ContextMenuItem::Item {
            name: name.into(),
            action,
            handler: Some(handler),
            os_action: None,
            checked: false,
            disabled: false,
        };
        let this = cx.weak_entity();

        vec![
            ContextMenuItem::Label {
                title: "File".into(),
                header: true,
            },
            item(
                "Rename",
                Some(Box::new(Rename)),
                Self::handler("Picked Rename", this.clone(), false),
            ),
            item(
                "Move to Trash",
                Some(Box::new(Trash)),
                Self::handler("Picked Move to Trash", this.clone(), false),
            ),
            ContextMenuItem::Separator,
            ContextMenuItem::Item {
                name: "Show Hidden Files".into(),
                action: Some(Box::new(ToggleHiddenFiles)),
                handler: Some(Self::handler(
                    "Toggled Show Hidden Files",
                    this.clone(),
                    true,
                )),
                os_action: None,
                checked: self.show_hidden_files,
                disabled: false,
            },
            ContextMenuItem::Item {
                name: "Disabled Item".into(),
                action: None,
                handler: None,
                os_action: None,
                checked: false,
                disabled: true,
            },
            ContextMenuItem::Separator,
            ContextMenuItem::Submenu {
                name: "Copy As".into(),
                items: vec![
                    item(
                        "Path",
                        None,
                        Self::handler("Picked Copy As > Path", this.clone(), false),
                    ),
                    item(
                        "Relative Path",
                        None,
                        Self::handler("Picked Copy As > Relative Path", this, false),
                    ),
                ],
            },
        ]
    }

    fn show_menu(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let options = ContextMenuOptions {
            anchor_rect: Bounds::new(event.position, Size::default()),
            anchor: PopupAnchor::TopLeft,
            offset: Point::default(),
        };
        let items = self.menu_items(cx);

        match window.show_context_menu(items, options, cx) {
            Ok(receiver) => {
                self.status = "Menu open".into();
                cx.notify();
                cx.spawn(async move |this, cx| {
                    let outcome = receiver.await.unwrap_or(ContextMenuOutcome::Dismissed);
                    this.update(cx, |this, cx| {
                        if matches!(outcome, ContextMenuOutcome::Dismissed) {
                            this.status = "Menu dismissed without a pick".into();
                            cx.notify();
                        }
                    })
                })
                .detach_and_log_err(cx);
            }
            Err(error) => {
                self.status = format!("{error}").into();
                cx.notify();
            }
        }
    }
}

impl Render for ContextMenuExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .bg(gpui::white())
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(gpui::black())
            .on_mouse_down(MouseButton::Right, cx.listener(Self::show_menu))
            .child(
                div()
                    .text_sm()
                    .text_color(gpui::black().opacity(0.5))
                    .child("Right-click anywhere"),
            )
            .child(self.status.clone())
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        if !example_support::load_fonts(cx) {
            return;
        }

        cx.bind_keys([
            KeyBinding::new("secondary-r", Rename, None),
            KeyBinding::new("backspace", Trash, None),
            KeyBinding::new("delete", Trash, None),
            KeyBinding::new("secondary-shift-h", ToggleHiddenFiles, None),
        ]);
        cx.activate(true);
        let bounds = Bounds::centered(None, size(px(400.), px(240.)), cx);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| ContextMenuExample {
                    show_hidden_files: false,
                    status: "No menu shown yet".into(),
                })
            },
        ) {
            eprintln!("could not open the example window: {error}");
        }
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
