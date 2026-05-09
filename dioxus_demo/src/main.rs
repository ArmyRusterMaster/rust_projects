use dioxus::prelude::*;

fn main() {
    // Используем запуск именно из модуля TUI
    dioxus_tui::launch(app);
}

fn app(cx: Scope) -> Element {
    render! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            background_color: "red", // Поменял на красный для проверки

            h1 { "РАБОТАЕТ!" }
            p { "Dioxus TUI запущен в Termux" }
            p { "Нажми Ctrl+C для выхода" }
        }
    }
}

