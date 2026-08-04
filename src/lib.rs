use dioxus::prelude::*;

pub mod data;
pub use crate::data::compute;
mod report;
mod ui;
use report::Nbsp;
use ui::MainPart;

static TAILWIND: Asset = asset!("/assets/tailwind.css");
static MAIN_CSS: Asset = asset!("/assets/main.css");
static MOON: Asset = asset!("/assets/moon.svg");
static SUN: Asset = asset!("/assets/sun.svg");
static FAVICON: Asset = asset!("/assets/favicon.ico");
static GITHUB: Asset = asset!("/assets/github-mark.svg");
static TAILWINDCSS: Asset = asset!("/assets/tailwindcss-mark.svg");

#[component]
pub fn App() -> Element {
    // Lit les cookies présents dans le browser pour intialiser les entrées
    // Reconstitue la chaine donnée par document.cookie avec un map et un join.
    let future = use_resource(move || async move {
        let mut eval = document::eval(
            r#"dioxus.send((await cookieStore.getAll()).map(c => `${c.name}=${c.value}`).join("; "));"#,
        );
        eval.recv::<String>().await.unwrap()
    });
    match future.read_unchecked().as_ref() {
        // On les a obtenus => on affiche l'application
        Some(cookies) => rsx! {
            document::Title { "Successions" }
            document::Stylesheet { href: TAILWIND }
            document::Stylesheet { href: MAIN_CSS }
            Body {
                MainPart { cookies }
            }
        },
        // On ne les a pas encore obtenus => on affiche une page blanche.
        // Inutile de définir une page d'attente car dans les faits un refresh
        // provoque un clignotement fugitif.
        _ => rsx! {
            div {}
        },
    }
}

// Whole HTML body (header + main part + footer)
#[component]
fn Body(children: Element) -> Element {
    let mut dark = use_signal(|| false);
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    // Fixe les tailles des images pour éviter leur clignotement à l'ouverture de la page
    const WIDTH: u32 = 28;
    const HEIGHT: u32 = 28;

    rsx! {
        div {
            class: if dark() { "dark" } else { "" },
            class: "h-full w-full bg-blue-50 dark:bg-blue-800 text-blue-900 dark:text-white",
            class: "flex flex-col justify-between",
            // Background colors need to be duplicated to remove a white stripe when scrollbar appears
            div { class: "bg-blue-50 dark:bg-blue-800",
                header {
                    id: "header",
                    class: "bg-blue-100 dark:bg-blue-900",
                    class: "flex flex-row items-center justify-between",
                    div { class: "ml-2 flex flex-row items-center gap-1",
                        a {
                            class: "tooltip-bottom tooltip",
                            href: "https://dioxuslabs.com/",
                            img { width: WIDTH, height: HEIGHT, src: FAVICON }
                            span { class: "tooltip-text w-37!", "Conçu avec le framework Dioxus." }
                        }
                        a {
                            class: "tooltip-bottom tooltip",
                            href: "https://tailwindcss.com/",
                            img {
                                width: WIDTH,
                                height: HEIGHT,
                                src: TAILWINDCSS,
                            }
                            span { class: "tooltip-text w-37!", "Styles rendus avec Tailwind CSS." }
                        }
                        a {
                            class: "tooltip-right tooltip",
                            href: "https://github.com/Thierry61/successions",
                            img {
                                class: " dark:invert",
                                width: WIDTH,
                                height: HEIGHT,
                                src: GITHUB,
                            }
                            span { class: "tooltip-text w-40!", "Code source déposé dans GitHub." }
                        }
                    }
                    span { class: "font-bold",
                        "Simulation de successions"
                        span { class: "hidden xs:inline", " - {PKG_VERSION}" }
                    }
                    // TODO: ajouter un bouton pour activer/désactiver les tooltips
                    button {
                        class: "rounded-md border border-blue-400 bg-blue-50 dark:bg-blue-600 px-3 py-1 m-2",
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
                    "Simulation non contractuelle et sans garanties"
                    Nbsp {}
                    "!"
                }
            }
        }
    }
}
