use dioxus::prelude::*;

use crate::data::{BeneficiaireState, BeneficiaireStateStoreExt, HeritierStateStoreExt, InputState, InputStateStoreExt, OptionState, OptionStateStoreExt, PremierDecesStoreExt, ResultState, ResultStateStoreExt};

// Formate un nombre avec des blancs comme séparateurs de milliers
pub fn format_num(val: ReadSignal<i32>) -> String {
    val.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(" ")  // séparateur
}

// Formate un nombre en lui ajoutant le symbol € et des blancs comme séparateurs de milliers
// et inclut le résultat dans un élement li
#[component]
fn Euros (val: ReadSignal<i32>, class: Option<&'static str>) -> Element {
    let num = format_num(val);
    let class = class.unwrap_or_default();
    rsx! {
        li { class: "text-right", class: "{class}",
            span { "{num} €" }
        }
    }
}

// Formate un nombre non monétaire. Initialement en réservant la place pour le symbol € sans l'afficher
// pour aligner les chiffres avec ceux des valeurs en euros, mais en fait le résultat est moyen.
// Pour réinstituer cette fonction il faut décommenter le span en commentaire.
#[component]
fn Nb (num: ReadSignal<i32>) -> Element {
    rsx! {
        li { class: "text-right",
            span {
                "{num}"
                        // span { class: "opacity-0", " €" }
            }
        }
    }
}

// Affiche la distribution des assurances-vie aux bénéficiaires
// (enfants, éventuellement survivant et total)
// TODO: Trouver un moyen d'utiliser une Option<Store<BeneficiaireState>> pour le survivant
#[component]
fn AssuranceVie(enfant: Store<BeneficiaireState>, survivant: Store<BeneficiaireState>, affiche_survivant: bool, total: Store<BeneficiaireState>) -> Element {
    rsx! {
        div {
            div { class: "flex flex-row gap-6",
                ul { class: "ml-5 list-disc list-outside",
                    li { class: "list-none text-right opacity-0", "Bénéficiaire : " }
                    li { "Capitaux décès bruts :" }
                    li { "Abattement :" }
                    li { "Part taxable :" }
                    li { class: "font-semibold text-red-600 dark:text-red-400", "Prélèvements :" }
                    li { class: "font-semibold text-green-600 dark:text-green-400",
                        "Capitaux décès nets :"
                    }
                }
                ul {
                    li { class: "text-center",
                        div { "Chaque enfant" }
                    }
                    Euros { val: enfant.brut() }
                    Euros { val: enfant.abattement() }
                    Euros { val: enfant.taxable() }
                    Euros {
                        class: "font-semibold text-red-600 dark:text-red-400",
                        val: enfant.prelevement(),
                    }
                    Euros {
                        class: "font-semibold text-green-600 dark:text-green-400",
                        val: enfant.net(),
                    }
                }
                if affiche_survivant {
                    ul {
                        li { class: "text-center",
                            div { "Survivant" }
                        }
                        Euros { val: survivant.brut() }
                        Euros { val: survivant.abattement() }
                        Euros { val: survivant.taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: survivant.prelevement(),
                        }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: survivant.net(),
                        }
                    }
                }
                ul {
                    li { class: "text-center",
                        div { "Total" }
                    }
                    Euros { val: total.brut() }
                    Euros { val: total.abattement() }
                    Euros { val: total.taxable() }
                    Euros {
                        class: "font-semibold text-red-600 dark:text-red-400",
                        val: total.prelevement(),
                    }
                    Euros {
                        class: "font-semibold text-green-600 dark:text-green-400",
                        val: total.net(),
                    }
                }
            }
        }
    }    
}

// Affichage des résultat pour une des 4 options possibles
#[component]
fn OptionChoisie(option: Store<OptionState>) -> Element {
    rsx! {
        div { class: "px-2 text-sm leading-6 text-gray-600 dark:text-white",
            h1 { class: "font-bold mt-2", "Répartition 1er décès :" }
            div {
                div { class: "flex flex-row gap-6",
                    ul { class: "ml-5 list-disc list-outside",
                        li { class: "list-none text-right opacity-0", "Héritier : " }
                        li { "Héritage en pleine propriété :" }
                        li { "Héritage en nue propriété :" }
                        li { "Héritage en usufruit :" }
                        li { "Part civile :" }
                        li { "Part fiscale :" }
                        li { "Abattement :" }
                        li { "Part taxable :" }
                        li { class: "font-semibold text-red-600 dark:text-red-400",
                            "Droits de succession :"
                        }
                        li { "Héritage net :" }
                        li { class: "font-semibold text-green-600 dark:text-green-400",
                            "Flux financier :"
                        }
                        li { "Flux financier avec AV:" }
                    }
                    ul {
                        li { class: "text-center",
                            div { "Chaque enfant" }
                        }
                        Euros { val: option.premier_enfant().heritage_pp() }
                        Euros { val: option.premier_enfant().heritage_np() }
                        Euros { val: option.premier_enfant().heritage_us() }
                        Euros { val: option.premier_enfant().part_civile() }
                        Euros { val: option.premier_enfant().part_fiscale() }
                        Euros { val: option.premier_enfant().abattement() }
                        Euros { val: option.premier_enfant().taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: option.premier_enfant().droits_succession(),
                        }
                        Euros { val: option.premier_enfant().heritage_net() }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: option.premier_enfant().flux_financier(),
                        }
                        Euros { val: option.premier_enfant().flux_financier_avec_av() }
                    }
                    ul {
                        li { class: "text-center",
                            div { "Survivant" }
                        }
                        Euros { val: option.premier_survivant().heritage_pp() }
                        Euros { val: option.premier_survivant().heritage_np() }
                        Euros { val: option.premier_survivant().heritage_us() }
                        Euros { val: option.premier_survivant().part_civile() }
                        Euros { val: option.premier_survivant().part_fiscale() }
                        Euros { val: option.premier_survivant().abattement() }
                        Euros { val: option.premier_survivant().taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: option.premier_survivant().droits_succession(),
                        }
                        Euros { val: option.premier_survivant().heritage_net() }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: option.premier_survivant().flux_financier(),
                        }
                        Euros { val: option.premier_survivant().flux_financier_avec_av() }
                    }
                    ul {
                        li { class: "text-center",
                            div { "Total" }
                        }
                        Euros { val: option.premier_total().heritage_pp() }
                        Euros { val: option.premier_total().heritage_np() }
                        Euros { val: option.premier_total().heritage_us() }
                        Euros { val: option.premier_total().part_civile() }
                        Euros { val: option.premier_total().part_fiscale() }
                        Euros { val: option.premier_total().abattement() }
                        Euros { val: option.premier_total().taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: option.premier_total().droits_succession(),
                        }
                        Euros { val: option.premier_total().heritage_net() }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: option.premier_total().flux_financier(),
                        }
                        Euros { val: option.premier_total().flux_financier_avec_av() }
                    }
                }
            }
            h1 { class: "font-bold mt-2", "Répartition 2eme décès :" }
            div {
                div { class: "flex flex-row gap-6",
                    ul { class: "ml-5 list-disc list-outside",
                        li { class: "list-none text-right opacity-0", "Héritier : " }
                        li { "Extinction usufruit :" }
                        li { "Part civile :" }
                        li { "Part fiscale :" }
                        li { "Abattement :" }
                        li { "Part taxable :" }
                        li { class: "font-semibold text-red-600 dark:text-red-400",
                            "Droits de succession :"
                        }
                        li { class: "font-semibold text-green-600 dark:text-green-400",
                            "Héritage net = Flux financier :"
                        }
                        li { "Flux financier avec AV:" }
                        li { "Flux financier total des 2 décès:" }
                    }
                    ul {
                        li { class: "text-center",
                            div { "Chaque enfant" }
                        }
                        Euros { val: option.deuxieme_enfant().extinction_us() }
                        Euros { val: option.deuxieme_enfant().part_civile() }
                        Euros { val: option.deuxieme_enfant().part_fiscale() }
                        Euros { val: option.deuxieme_enfant().abattement() }
                        Euros { val: option.deuxieme_enfant().taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: option.deuxieme_enfant().droits_succession(),
                        }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: option.deuxieme_enfant().flux_financier(),
                        }
                        Euros { val: option.deuxieme_enfant().flux_financier_avec_av() }
                        Euros { val: option.cumul_enfant() }
                    }
                    ul {
                        li { class: "text-center",
                            div { "Total" }
                        }
                        Euros { val: option.deuxieme_total().extinction_us() }
                        Euros { val: option.deuxieme_total().part_civile() }
                        Euros { val: option.deuxieme_total().part_fiscale() }
                        Euros { val: option.deuxieme_total().abattement() }
                        Euros { val: option.deuxieme_total().taxable() }
                        Euros {
                            class: "font-semibold text-red-600 dark:text-red-400",
                            val: option.deuxieme_total().droits_succession(),
                        }
                        Euros {
                            class: "font-semibold text-green-600 dark:text-green-400",
                            val: option.deuxieme_total().flux_financier(),
                        }
                        Euros { val: option.deuxieme_total().flux_financier_avec_av() }
                        Euros { val: option.cumul_total() }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Rapport(snapshot: Store<InputState>, result: Store<ResultState>, show_report: ReadSignal<bool>) -> Element {
    rsx! {
        details {
            class: "text-gray-900 dark:text-white",
            class: if show_report() { " block" } else { "hidden" },
            open: "true",
            summary { class: "m-2 text-sm leading-6 font-semibold select-none", "Détails du calcul :" }
            div { class: "px-2 text-sm leading-6 text-gray-600 dark:text-white",
                h1 { class: "font-bold", "Données d'entrée :" }
                div { class: "w-160 grid grid-cols-2 gap-2 justify-items-start",
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",
                            li { "Nombre d'enfants :" }
                            li { "Résidence principale :" }
                            li { "Placements hors AV et PER :" }
                            li { "Dettes et impôts restant dus :" }
                        }
                        ul {
                            Nb { num: snapshot.nb_enfants() }
                            Euros { val: snapshot.residence_principale() }
                            Euros { val: snapshot.placements() }
                            Euros { val: snapshot.dettes() }
                        }
                    }
                    div { class: "flex flex-row gap-4",
                        ul { class: "ml-5 list-disc list-outside",
                            li { "Biens meublants :" }
                            li { "Frais funéraires réels :" }
                            li { "Donations-partages :" }
                        }
                        ul {
                            Euros { val: snapshot.biens_meublants() }
                            Euros { val: snapshot.frais_funeraires() }
                            Euros { val: snapshot.donations_partages() }
                        }
                    }
                }
                br {}
                div {
                    div { class: "flex flex-row gap-6",
                        ul { class: "ml-5 list-disc list-outside",
                            li { class: "list-none text-right opacity-0", "Epoux : " }
                            li { "Age des époux :" }
                            li { "AV au bénéfice du conjoint :" }
                            li { "AV au bénéfice des enfants :" }
                            li { "PER au bénéfice du conjoint :" }
                        }
                        ul {
                            li { class: "text-center", "Vous" }
                            Nb { num: snapshot.age_vous() }
                            Euros { val: snapshot.av_vous_conjoint() }
                            Euros { val: snapshot.av_vous_enfants() }
                            Euros { val: snapshot.per_vous_conjoint() }
                        }
                        ul {
                            li { class: "text-center", "Conjoint" }
                            Nb { num: snapshot.age_conjoint() }
                            Euros { val: snapshot.av_conjoint_conjoint() }
                            Euros { val: snapshot.av_conjoint_enfants() }
                            Euros { val: snapshot.per_conjoint_conjoint() }
                        }
                    }
                }
                br {}
                div { class: "w-160 grid grid-cols-2 gap-2 justify-items-start",
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
            div { class: "px-2 pt-2 text-sm leading-6 text-gray-600 dark:text-white",
                h1 { class: "font-bold", "Succession 1er décès :" }
                div {
                    div { class: "flex flex-row gap-6",
                        ul { class: "ml-5 list-disc list-outside",
                            li { class: "list-none text-right opacity-0", "Plan : " }
                            li { "Actif brut de communauté :" }
                            li { "Récompense due par le survivant :" }
                            li { "Récompense due par le défunt :" }
                            li { "Actif net de communauté :" }
                            li { "Solde de récompenses :" }
                            li { "Actif net de communauté après récompenses :" }
                            li { "Actif brut de succession :" }
                            li { "Actif net de succession :" }
                            li { "Part du survivant hors succession :" }
                        }
                        ul {
                            li { class: "text-center", "Civil" }
                            Euros { val: result.premier_deces_civil().actif_brut_communaute() }
                            Euros { val: result.premier_deces_civil().recompense_due_par_le_survivant() }
                            Euros { val: result.premier_deces_civil().recompense_due_par_le_defunt() }
                            Euros { val: result.premier_deces_civil().actif_net_communaute() }
                            Euros { val: result.premier_deces_civil().solde_recompenses() }
                            Euros { val: result.premier_deces_civil().actif_net_communaute_ajuste() }
                            Euros { val: result.premier_deces_civil().actif_brut_succession() }
                            Euros { val: result.premier_deces_civil().actif_net_succession() }
                            Euros { val: result.premier_deces_civil().part_survivant_hors_succession() }
                        }
                        ul {
                            li { class: "text-center", "Fiscal" }
                            Euros { val: result.premier_deces_fiscal().actif_brut_communaute() }
                            Euros { val: result.premier_deces_fiscal().recompense_due_par_le_survivant() }
                            Euros { val: result.premier_deces_fiscal().recompense_due_par_le_defunt() }
                            Euros { val: result.premier_deces_fiscal().actif_net_communaute() }
                            Euros { val: result.premier_deces_fiscal().solde_recompenses() }
                            Euros { val: result.premier_deces_fiscal().actif_net_communaute_ajuste() }
                            Euros { val: result.premier_deces_fiscal().actif_brut_succession() }
                            Euros { val: result.premier_deces_fiscal().actif_net_succession() }
                            Euros { val: result.premier_deces_fiscal().part_survivant_hors_succession() }
                        }
                    }
                }
            }
            div { class: "px-2 pt-2 text-sm leading-6 text-gray-600 dark:text-white",
                h1 { class: "font-bold", "Assurances-vie 1er décès :" }
                AssuranceVie {
                    enfant: result.premier_av_enfant(),
                    survivant: result.premier_av_survivant(),
                    affiche_survivant: true,
                    total: result.premier_av_total(),
                }
            }
            div { class: "px-2 pt-2 text-sm leading-6 text-gray-600 dark:text-white",
                h1 { class: "font-bold", "Assurances-vie 2ème décès :" }
                AssuranceVie {
                    enfant: result.deuxieme_av_enfant(),
                    // Store bidon pour l'AV non affichée
                    survivant: result.deuxieme_av_enfant(),
                    affiche_survivant: false,
                    total: result.deuxieme_av_total(),
                }
            }
            details { class: "p-2", open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option totalité en usufruit :"
                }
                OptionChoisie { option: result.option_totalite_us() }
            }
            details { class: "p-2", open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option 1/4 en pleine propriété :"
                }
                OptionChoisie { option: result.option_1_4_pp() }
            }
            details { class: "p-2", open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option 1/4 en pleine propriété - 3/4 en usufruit :"
                }
                OptionChoisie { option: result.option_1_4_pp_3_4_us() }
            }
            details { class: "p-2", open: "false",
                summary { class: "text-sm leading-6 font-semibold text-gray-900 dark:text-white select-none",
                    "Option quotité disponible en pleine propriété :"
                }
                OptionChoisie { option: result.option_qd_pp() }
            }
        }
    }
}
