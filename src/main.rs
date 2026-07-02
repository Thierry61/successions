use dioxus::prelude::*;

mod data;
mod report;
mod ui;
use ui::MainPart;

static TAILWIND: Asset = asset!("/assets/tailwind.css");
static MOON: Asset = asset!("/assets/moon.svg");
static SUN: Asset = asset!("/assets/sun.svg");  

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND }
        Body { MainPart {} }
    }
}

// Whole HTML body (header + main part + footer)
#[component]
fn Body(children: Element) -> Element {
    let mut dark = use_signal(|| false);

    rsx! {
        div {
            class: if dark() { "dark" } else { "" },
            class: "h-full w-full bg-blue-50 dark:bg-blue-700 text-blue-900 dark:text-white",
            class: "flex flex-col justify-between",
            // Background colors need to be duplicated to remove a white stripe when scrollbar appears
            div { class: "bg-blue-50 dark:bg-blue-700",
                header {
                    id: "header",
                    class: "bg-blue-100 dark:bg-blue-900",
                    class: "flex flex-row items-center justify-between",
                    span { class: "m-3 font-semibold", "Simulation de successions" }
                    // TODO: ajouter un bouton pour activer/désactiver les tooltips
                    button {
                        class: "rounded-md border border-blue-400 bg-blue-50 dark:bg-blue-500 px-3 py-1 m-3",
                        class: "tooltip-left tooltip",
                        onclick: move |_| dark.toggle(),
                        img {
                            class: "w-5 h-5 dark:invert",
                            src: if dark() { MOON } else { SUN },
                        }
                        span { class: "tooltip-text",
                            {format!("Basculer vers le thème {}", if dark() { "clair" } else { "sombre" })}
                        }
                    }
                }
                {children}
            }
            footer {
                id: "footer",
                class: "w-full bg-blue-100 dark:bg-blue-900 justify-self-end",
                class: "flex flex-row items-center justify-between",
                span { class: "m-3 font-semibold",
                    "Simulation non contractuelle et sans aucunes garanties !"
                }
            }
        }
    }
}
