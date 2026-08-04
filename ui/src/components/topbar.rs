use dioxus::prelude::*;

use crate::components::atoms::{Icon, IconKind};
use crate::storage;

#[component]
pub fn Topbar() -> Element {
    let mut theme = use_signal(|| storage::load_theme().unwrap_or_else(|| "light".to_string()));
    let is_dark = theme.read().as_str() == "dark";

    let toggle_theme = move |_| {
        let next = if theme.peek().as_str() == "dark" {
            "light"
        } else {
            "dark"
        };
        storage::save_theme(next);
        storage::apply_theme(next);
        theme.set(next.to_string());
    };

    let theme_icon = if is_dark { IconKind::Sun } else { IconKind::Moon };

    rsx! {
        header { class: "topbar",
            div { class: "topbar__brand",
                div { class: "topbar__logo", "C" }
                span { class: "topbar__brand-name", "Certbot API" }
            }
            div { class: "topbar__actions",
                button {
                    class: "topbar__icon-btn",
                    title: "Toggle theme",
                    onclick: toggle_theme,
                    Icon { kind: theme_icon }
                }
            }
        }
    }
}
