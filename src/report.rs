use dioxus::prelude::*;

use crate::data::{InputState, InputStateStoreExt, ResultState, ResultStateStoreExt};

// Formate un nombre en lui ajoutant le symbol € et des blancs comme séparateur de milliers
#[component]
fn Euros (val: ReadSignal<i32>) -> Element {
    let num = val.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(" ");  // séparateur
    rsx! {
        span { "{num} €" }
    }
}

// Formate un nombre en réservant la place pour le symbol € sans l'afficher
// (pour allgner les chiffres avec ceux des valeurs en euros)
#[component]
fn Nb (num: ReadSignal<i32>) -> Element {
    rsx! {
        span {
            "{num}"
            span { class: "opacity-0", " €" }
        }
    }
}

#[component]
pub fn Rapport(snapshot: Store<InputState>, result: Store<ResultState>, show_report: ReadSignal<bool>) -> Element {
    rsx! {
        details {
            class: "border-y border-transparent open:border-black/10 open:bg-gray-100 dark:open:bg-gray-600 text-gray-900 dark:text-white",
            class: if show_report() { " block" } else { "hidden" },
            open: "false",
            summary { class: "m-2 text-sm leading-6 font-semibold select-none",
                "Détails du dernier calcul :"
            }
            div { class: "px-2 text-sm leading-6 text-gray-600 dark:text-white",
                h1 { class: "font-bold", "Données d'entrée :" }
                div { class: "w-170 grid grid-cols-2 gap-2 justify-items-start",
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",

                            li { "Nombre d'enfants :" }
                            li { "Résidence principale :" }
                            li { "Placements hors AV et PER :" }
                            li { "Dettes et impôts restant dus :" }
                        }
                        ul {
                            li { class: "text-right",
                                Nb { num: snapshot.nb_enfants() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.residence_principale() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.placements() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.dettes() }
                            }
                        }
                    }
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",

                            li { "Biens meublants :" }
                            li { "Frais funéraires réels :" }
                            li { "Donations-partages :" }
                        }
                        ul {
                            li { class: "text-right",
                                Euros { val: snapshot.biens_meublants() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.frais_funeraires() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.donations_partages() }
                            }
                        }
                    }
                }
                br {}
                div { class: "",
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",
                            li { "Age des époux :" }
                            li { "AV au bénéfice du conjoint :" }
                            li { "AV au bénéfice des enfants :" }
                            li { "PER bénéfice du conjoint :" }
                        }
                        ul {
                            li { "vous :" }
                            li { "vous :" }
                            li { "vous :" }
                            li { "vous :" }
                        }
                        ul {
                            li { class: "text-right",
                                Nb { num: snapshot.age_vous() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.av_vous_conjoint() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.av_vous_enfants() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.per_vous_conjoint() }
                            }
                        }
                        ul {
                            li { "votre conjoint :" }
                            li { "votre conjoint :" }
                            li { "votre conjoint :" }
                            li { "votre conjoint :" }
                        }
                        ul {
                            li { class: "text-right",
                                Nb { num: snapshot.age_conjoint() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.av_conjoint_conjoint() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.av_conjoint_enfants() }
                            }
                            li { class: "text-right",
                                Euros { val: snapshot.per_conjoint_conjoint() }
                            }
                        }
                    }
                }
                br {}
                div { class: "w-170 grid grid-cols-2 gap-2 justify-items-start",
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",

                            li {
                                "Forfait biens mobiliers "
                                if !*snapshot.forfait_mobilier().read() {
                                    "non "
                                }
                                "utilisé."
                            }
                            li {
                                "Ordres des décès : "
                                if *snapshot.ordre_deces().read() {
                                    "vous puis votre conjoint."
                                } else {
                                    "votre conjoint puis vous."
                                }
                            }
                        }
                    }
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",

                            li {
                                "Dispense de récompense "
                                if !*snapshot.dispense_recompense().read() {
                                    "non "
                                }
                                "demandée."
                            }
                            li {
                                "Coûts de partage "
                                if *snapshot.ignorer_couts_partage().read() {
                                    "ignorés."
                                } else {
                                    "pris en compte."
                                }
                            }
                        }
                    }
                }
            }
            div { class: "p-2",
                h1 { class: "font-semibold", "Calcul succession :" }
                div { class: "", "TODO" }
            }
            details {
                class: "p-2 border-y border-transparent open:border-black/10 open:bg-gray-100 dark:open:bg-gray-600",
                open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option totalité en usufruit :"
                }
                div { "TODO" }
            }
            details {
                class: "p-2 border-y border-transparent open:border-black/10 open:bg-gray-100 dark:open:bg-gray-600",
                open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option 1/4 en pleine propriété :"
                }
                div { "TODO" }
            }
            details {
                class: "p-2 border-y border-transparent open:border-black/10 open:bg-gray-100 dark:open:bg-gray-600",
                open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option 1/4 en pleine propriété - 3/4 en usufruit :"
                }
                div { "TODO" }
            }
            details {
                class: "p-2 border-y border-transparent open:border-black/10 open:bg-gray-100 dark:open:bg-gray-600",
                open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option quotité disponible en pleine propriété :"
                }
                div { "TODO" }
            }
        }
    }
}
