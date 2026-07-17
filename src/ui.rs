use dioxus::prelude::*;

use crate::data::{
    calcul_biens_meublants, HeritierStateStoreExt, InputState, InputStateStoreExt,
    OptionStateStoreExt, ResultState, ResultStateStoreExt, DEFAUT_NB_ENFANTS,
};
use crate::report::{format_num, Rapport};

// Gestion d'un fieldset:
// - la légende peut être centrée ou alignée à gauche
// - une partie de la légende peut être masquée quand l'écran est petit
#[component]
fn Fieldset(
    legend: &'static str,
    optional: &'static str,
    center: bool,
    children: Element,
) -> Element {
    rsx! {
        fieldset {
            class: "bg-blue-100 dark:bg-blue-600 border-t border-l border-r border-blue-300 dark:border-blue-800",
            class: if !center { "border rounded-lg drop-shadow-md drop-shadow-md" },
            class: if center { "rounded-t-lg" },
            legend {
                class: "font-semibold",
                class: if center { "text-center" } else { "ml-3" },
                div {
                    span { "{legend}" }
                    if !optional.is_empty() {
                        span { class: "hidden md:inline", " {optional}" }
                    }
                }
            }
            {children}
        }
    }
}

#[component]
fn Checkbox(
    id: &'static str,
    lab: &'static str,
    tooltip: &'static str,
    signal: WriteSignal<bool>,
    store: Option<Store<InputState>>,
) -> Element {
    rsx! {
        div { class: "tooltip-top tooltip",
            span { class: "tooltip-text ml-12!", {tooltip} }
            input {
                id,
                class: "mx-2 my-1 accent-blue-50 dark:accent-blue-700",
                r#type: "checkbox",
                onclick: move |_| {
                    signal.toggle();
                    // Recalcule le champ biens meublants si forfait mobilier est coché.
                    gere_biens_meublants(store);
                },
                checked: signal,
            }
            label { r#for: id, "{lab}" }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum InputType {
    NbEnfants,
    BiensMeublants,
    ResidencePrincipale,
    Placements,
    Autres,
}

#[component]
fn Input(
    signal: WriteSignal<i32>,
    store: Option<Store<InputState>>,
    input_type: InputType,
) -> Element {
    // Désactive le champ biens meublants quand forfait mobilier est coché
    let disabled = use_memo(move || {
        if input_type == InputType::BiensMeublants {
            if let Some(store) = store {
                return *store.forfait_mobilier().read();
            };
        };
        false
    });
    // Traitement des événements oninput et onchange. La différence entre les 2
    // est que le premier ne supprime pas une série de 0 à gauche.
    let mut manage_input_and_change = move |e: Event<FormData>, is_change: bool| {
        if !e.valid() && !is_change {
            e.prevent_default();
            return;
        }
        // Récupère la valeur saisie
        let new_val = e.value();
        // Stocke la nouvelle valeur saisie sauf en mode input si le premier chiffre à gauche vaut 0
        // pour que l'utilisateur se soit pas supris de voir disparaitre une série de 0 à gauche
        // alors qu'il voulait juste remplacer le chiffre le plus signicatif (par exemple il voulait
        // remplacer 30000 par 40000, en effacant le 3 et en tapant 4 à la place).
        if !is_change && new_val.starts_with('0') {
            return;
        }
        // On met la valeur par défaut à la place d'un champ vide en mode onchange
        if new_val.is_empty() && is_change {
            signal.set(if input_type == InputType::NbEnfants {
                DEFAUT_NB_ENFANTS
            } else {
                i32::default()
            });
        } else {
            // Le unwrap_or remet la valeur courante si la nouvelle valeur est invalide ou négative
            let unsigned_old_val = signal() as u32;
            let mut unsigned_new_val: u32 = new_val.parse().unwrap_or(unsigned_old_val);
            // Idem si le nb d'enfants vaut 0
            if input_type == InputType::NbEnfants && unsigned_new_val == 0 {
                unsigned_new_val = unsigned_old_val;
            }
            *signal.write() = unsigned_new_val as i32;
        }
        // Puis effectue éventuellement un traitement global inter-champs
        if input_type == InputType::ResidencePrincipale || input_type == InputType::Placements {
            gere_biens_meublants(store);
        }
    };
    // Vérifie le champ caractère par caractère
    let manage_input = move |e: Event<FormData>| {
        manage_input_and_change(e, false);
    };
    // Vérifie le champ à la fin de la saisie
    let manage_change = move |e: Event<FormData>| {
        manage_input_and_change(e, true);
    };
    rsx! {
        input {
            class: "w-17 h-5 m-1 pr-1 text-end bg-blue-50 dark:bg-blue-500 rounded-sm",
            class: "disabled:bg-gray-300 dark:disabled:bg-gray-500",
            class: "remove-arrow",
            r#type: "number",
            min: if input_type == InputType::NbEnfants { "1" } else { "0" },
            pattern: "[0-9]+",
            disabled,
            // Les 2 sont nécessaires pour gérer correctement le double effacement du dernier caractère.
            oninput: manage_input,
            onchange: manage_change,
            value: signal,
        }
    }
}

#[component]
fn Output(signal: ReadSignal<i32>) -> Element {
    let num = format_num(signal);
    rsx! {
        input {
            class: "w-18 h-5 m-1 pr-1 text-end bg-blue-50 dark:bg-blue-500 rounded-sm ml-2",
            class: "disabled:bg-gray-300 dark:disabled:bg-gray-500",
            class: "remove-arrow",
            disabled: true,
            value: num,
        }
    }
}

#[component]
fn InputWithLabel(
    id: &'static str,
    lab: &'static str,
    tooltip: &'static str,
    signal: WriteSignal<i32>,
    store: Option<Store<InputState>>,
    input_type: InputType,
) -> Element {
    rsx! {
        div { id,
            div {
                class: "w-48 px-2 py-1 flex flex-row justify-between bg-blue-100 dark:bg-blue-600 rounded-lg drop-shadow-md",
                class: "border border-blue-300 dark:border-blue-800",
                div { class: if !tooltip.is_empty() { "tooltip-top tooltip" },
                    span { class: "tooltip-text", {tooltip} }
                    {lab}
                }
                Input { signal, store, input_type }
            }
        }
    }
}

#[component]
fn InputWithoutLabel(id: &'static str, signal: WriteSignal<i32>) -> Element {
    rsx! {
        div { id,
            Input { signal, input_type: InputType::Autres }
        }
    }
}

// Si forfait mobilier est coché alors on maintient dans biens meublants la valeur 5% de l'actif brut successoral en permanence
fn gere_biens_meublants(store: Option<Store<InputState>>) {
    if let Some(store) = store {
        let forfait_mobilier = *store.forfait_mobilier().read();
        if forfait_mobilier {
            let residence_principale = *store.residence_principale().read();
            let placements = *store.placements().read();
            let dettes = *store.dettes().read();
            store.biens_meublants().set(calcul_biens_meublants(
                residence_principale,
                placements,
                dettes,
            ));
        }
    }
}

#[component]
pub fn MainPart(cookies: String) -> Element {
    // Inputs et options
    let input = use_store(|| InputState::new_from_cookies(&cookies));
    let snapshot = use_store(InputState::new);
    // Outputs
    let result = use_store(ResultState::default);
    // Petite animation quand l'utilisateur click sur "Calculer"
    let mut animate_click = use_signal(|| false);
    // Affiche le rapport dès qu'un calcul a été lancé
    let mut show_report = use_signal(|| false);

    rsx! {
        // Décommenter la ligne suivante pour debugger les cookies
        // "Cookies: {cookies}"
        // Une forme est nécessaire pour déclencher le calcul en entrant un retour-chariot sur n'importe quel champ.
        form {
            div { class: "m-3",
                "Hypothèses principales :"
                ul { class: "ml-5 list-disc list-outside",
                    li {
                        "couple marié sous le régime légal (communauté réduite aux acquêts) avec au moins un enfant"
                    }
                    li {
                        "tous les éléments sont communs (biens, dettes et fonds alimentant les placements et les donations)"
                    }
                    li { "les versements sur les assurances-vie ont été effectués avant 70 ans" }
                    li {
                        "les bénéficiaires des assurances-vie sont soit les enfants, soit le conjoint (puis les enfants au second décès)"
                    }
                    li {
                        "les bénéficiaires des PER sont le conjoint (puis les enfants au second décès)"
                    }
                }
            }
            div { id: "inputs", class: "m-2 text-sm flex flex-wrap gap-4",
                InputWithLabel {
                    id: "nb-enfants",
                    lab: "Nombre d'enfants",
                    tooltip: "Nombre d'enfants communs du couple, doit être supérieur ou égal à 1.",
                    signal: input.nb_enfants(),
                    input_type: InputType::NbEnfants,
                }
                InputWithLabel {
                    id: "RP",
                    lab: "Résidence principale",
                    tooltip: "Pour abattement de 20% dans le calcul des droits (plan fiscal).",
                    signal: input.residence_principale(),
                    store: Some(input),
                    input_type: InputType::ResidencePrincipale,
                }
                InputWithLabel {
                    id: "placements",
                    lab: "Placements hors AV/PER",
                    tooltip: "Placements sauf AV et PER qui ont une fiscalité spécifique et une éventuelle récompense à prendre en compte.",
                    signal: input.placements(),
                    store: Some(input),
                    input_type: InputType::Placements,
                }
                InputWithLabel {
                    id: "dettes",
                    lab: "Dettes et impôts",
                    tooltip: "Dettes de la communauté, y compris les impôts restants à payer.",
                    signal: input.dettes(),
                    input_type: InputType::Autres,
                }
                InputWithLabel {
                    id: "biens-meublants",
                    lab: "Biens meublants",
                    tooltip: "Intégrés dans l'actif successoral uniquement sur le plan fiscal si forfait mobilier ou sur les 2 plans (fiscal et civil) sinon",
                    signal: input.biens_meublants(),
                    store: Some(input),
                    input_type: InputType::BiensMeublants,
                }
                InputWithLabel {
                    id: "frais-funeraires",
                    lab: "Frais funéraires réels",
                    tooltip: "Frais funéraire réels déduits de l'actif successoral net (plan civil), par opposition au forfait de 1500€ déduit sur le plan fiscal.",
                    signal: input.frais_funeraires(),
                    input_type: InputType::Autres,
                }
                InputWithLabel {
                    id: "donations-partages",
                    lab: "Donations partages",
                    tooltip: "Donations-partages de moins de 15 ans, conjonctives, égalitaires et hors dons Sarkozy (plan fiscal).",
                    signal: input.donations_partages(),
                    input_type: InputType::Autres,
                }
            }
            div { class: "ml-2 mb-2 flex flex-wrap gap-4",
                Fieldset {
                    legend: "Données du couple",
                    optional: "",
                    center: false,
                    div {
                        id: "données-couple",
                        class: "w-100 pl-2 pb-1 grid grid-cols-4",
                        div { class: "col-span-2", "" }
                        div { class: "pl-5 py-1", "Vous" }
                        div { class: "pl-2 py-1", "Conjoint" }
                        div { class: "col-span-2 tooltip-top tooltip",
                            span { class: "tooltip-text w-70!",
                                "Permet de déterminer le barème fiscal de l'usufruit et de la nue-propriété, ainsi que la fiscalité des PER."
                            }
                            "Ages des époux"
                        }
                        InputWithoutLabel { id: "age_vous", signal: input.age_vous() }
                        InputWithoutLabel { id: "age_conjoint", signal: input.age_conjoint() }
                        div { class: "col-span-2 tooltip-top tooltip",
                            span { class: "tooltip-text w-70!",
                                "AV défunt bénéfice survivant : aucune récompense n'est due."
                                br {}
                                "AV survivant : le survivant doit une récompense à la communauté."
                            }
                            "AV bénéfice conjoint"
                        }
                        InputWithoutLabel {
                            id: "av_vous_conjoint",
                            signal: input.av_vous_conjoint(),
                        }
                        InputWithoutLabel {
                            id: "av_conjoint_conjoint",
                            signal: input.av_conjoint_conjoint(),
                        }
                        div { class: "col-span-2 tooltip-top tooltip",
                            span { class: "tooltip-text w-66!",
                                "AV défunt bénéfice enfants : le défunt doit une récompense à la communauté (sauf si dispense)."
                                br {}
                                "AV survivant : le survivant doit une récompense à la communauté."
                            }
                            "AV bénéfice enfants"
                        }
                        InputWithoutLabel {
                            id: "av_vous_enfants",
                            signal: input.av_vous_enfants(),
                        }
                        InputWithoutLabel {
                            id: "av_conjoint_enfants",
                            signal: input.av_conjoint_enfants(),
                        }
                        div { class: "col-span-2 tooltip-top tooltip",
                            span { class: "tooltip-text w-75!",
                                "PER défunt bénéfice survivant : aucune récompense n'est due."
                                br {}
                                "PER survivant : aucune récompense n'est due."
                            }
                            "PER bénéfice conjoint"
                        }
                        InputWithoutLabel {
                            id: "per_vous_conjoint",
                            signal: input.per_vous_conjoint(),
                        }
                        InputWithoutLabel {
                            id: "per_conjoint_conjoint",
                            signal: input.per_conjoint_conjoint(),
                        }
                    }
                }
                Fieldset { legend: "Options", optional: "", center: false,
                    div { class: "w-100 py-1 grid grid-cols-1",
                        Checkbox {
                            id: "forfait-mobilier",
                            lab: "Forfait biens mobiliers",
                            tooltip: "Forfait de 5% de l'actif successoral brut pour les biens meublants.",
                            signal: input.forfait_mobilier(),
                            store: Some(input),
                        }
                        Checkbox {
                            id: "ordre-décès",
                            lab: "Ordre des décès : vous puis votre conjoint",
                            tooltip: "Simulation supposant que vous décédiez avant votre conjoint.",
                            signal: input.ordre_deces(),
                        }
                        Checkbox {
                            id: "dispense-récompense",
                            lab: "Dispense de récompense demandée par survivant",
                            tooltip: "Dispense de récompense demandée par le conjoint survivant pour les AV du défunt au bénéfice des enfants.",
                            signal: input.dispense_recompense(),
                        }
                        Checkbox {
                            id: "ignorer-couts-partage",
                            lab: "Ignorer les coûts de partage",
                            tooltip: "Ne pas calculer les coûts de partage (droits de partage et émoluments associés).",
                            signal: input.ignorer_couts_partage(),
                        }
                        Checkbox {
                            id: "ignorer-couts-partage",
                            lab: "Ignorer la déclaration de succession",
                            tooltip: "Ne pas calculer les émoluments de la déclaration de succession.",
                            signal: input.ignorer_declaration_succession(),
                        }
                    }
                }
                Fieldset { legend: "Résultats", optional: "", center: false,
                    div {
                        id: "résultats",
                        class: "md:px-2 px-0 pb-2 grid grid-cols-7 gap-x-0 md:gap-x-2 gap-y-0",
                        div { class: "mt-3",
                            button {
                                class: "px-3 py-2 font-bold bg-green-100 text-green-700 dark:bg-green-600 dark:text-white",
                                class: "border border-green-400 dark:border-white rounded-lg drop-shadow-md",
                                class: "transition duration-200",
                                class: if animate_click() { "-translate-y-1 scale-110" },
                                class: "tooltip tooltip-top",
                                ontransitionend: move |_| { animate_click.set(false) },
                                onclick: move |event| {
                                    animate_click.set(true);
                                    // Appel du traitement de calcul de la succession
                                    ResultState::store_compute(input, snapshot, result);
                                    // Affiche le rapport
                                    show_report.set(true);
                                    // Commande javascript sauvegardant les entrées dans des cookies
                                    let js = InputState::to_cookies(input);
                                    // Execution de cette commande javascript
                                    spawn(async move {
                                        let eval = document::eval(&js);
                                        let _ = eval.await;
                                    });
                                    // Evite le rechargement de la page provoqué par la forme
                                    event.prevent_default();
                                },
                                span { class: "tooltip-text",
                                    "Lance le calcul de la succession et mémorise les données d'entrée."
                                }
                                "Calculer"
                            }
                        }
                        div { class: "col-span-2",
                            Fieldset {
                                legend: "1er décès",
                                optional: "",
                                center: true,
                                div { class: "pl-2 grid grid-cols-2 items-stretch",
                                    div { class: "tooltip tooltip-top",
                                        span { class: "tooltip-text w-65!",
                                            "Valeur reçue en pleine-propriété par le conjoint survivant (hors usufruit), incluant les assurances-vie dont il est bénéficiaire."
                                        }
                                        "Conjoint"
                                        br {}
                                        "survivant"
                                    }
                                    div { class: "pl-1 tooltip tooltip-top",
                                        span { class: "tooltip-text w-65!",
                                            "Valeur reçue en pleine-propriété par chaque enfant (hors nue-propriété), incluant les assurances-vie dont il est bénéficiaire."
                                        }
                                        "Chaque"
                                        br {}
                                        "enfant"
                                    }
                                }
                            }
                        }
                        div {
                            Fieldset {
                                legend: "2ème",
                                optional: "décès",
                                center: true,
                                div { class: "pl-2 tooltip tooltip-top",
                                    span { class: "tooltip-text w-65!",
                                        "Valeur reçue en pleine-propriété par chaque enfant, incluant les assurances-vie dont il est bénéficiaire."
                                    }
                                    "Chaque"
                                    br {}
                                    "enfant"
                                }
                            }
                        }
                        div { class: "col-span-3",
                            Fieldset {
                                legend: "Cumul des 2 décès",
                                optional: "",
                                center: true,
                                div { class: "pl-2 grid grid-cols-3 items-end",
                                    div { class: "tooltip tooltip-top",
                                        span { class: "tooltip-text w-65!",
                                            "Valeur reçue en pleine-propriété par chaque enfant, incluant les assurances-vie dont il est bénéficiaire."
                                        }
                                        "Chaque"
                                        br {}
                                        "enfant"
                                    }
                                    div { class: "pl-1 tooltip tooltip-top",
                                        span { class: "tooltip-text", "Impôts perçus par l'Etat." }
                                        "Etat"
                                    }
                                    div { class: "pl-2 tooltip tooltip-top",
                                        span { class: "tooltip-text w-35!",
                                            "Emoluments perçus par le notaire."
                                        }
                                        "Notaire"
                                    }
                                }
                            }
                        }
                        div { class: "tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option totalité en usufruit choisie par le conjoint survivant."
                            }
                            "100% US"
                        }
                        div { class: "col-span-2 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_totalite_us().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_totalite_us().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-1 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_totalite_us().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-3 border-x border-blue-300 dark:border-blue-800 grid grid-cols-3 items-stretch",
                            Output { signal: result.option_totalite_us().cumul_enfant() }
                            Output { signal: result.option_totalite_us().cumul_etat() }
                            Output { signal: result.option_totalite_us().cumul_notaire() }
                        }
                        div { class: "tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option 1/4 en pleine propriété choisie par le conjoint survivant."
                            }
                            "¼ PP"
                        }
                        div { class: "col-span-2 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_1_4_pp().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_1_4_pp().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-1 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_1_4_pp().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-3 border-x border-blue-300 dark:border-blue-800 grid grid-cols-3 items-stretch",
                            Output { signal: result.option_1_4_pp().cumul_enfant() }
                            Output { signal: result.option_1_4_pp().cumul_etat() }
                            Output { signal: result.option_1_4_pp().cumul_notaire() }
                        }
                        div { class: "tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option 1/4 en pleine propriété et 3/4 en usufruit choisie par le conjoint survivant."
                            }
                            "¼ PP ¾ US"
                        }
                        div { class: "col-span-2 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_1_4_pp_3_4_us().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_1_4_pp_3_4_us().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-1 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_1_4_pp_3_4_us().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-3 border-x border-blue-300 dark:border-blue-800 grid grid-cols-3 items-stretch",
                            Output { signal: result.option_1_4_pp_3_4_us().cumul_enfant() }
                            Output { signal: result.option_1_4_pp_3_4_us().cumul_etat() }
                            Output { signal: result.option_1_4_pp_3_4_us().cumul_notaire() }
                        }
                        // Tooltip top au lieu de right pour éviter une bande blanche en bas
                        div { class: "tooltip-right tooltip",
                            span { class: "tooltip-text w-50!",
                                "Option quotité disponible en pleine propriété choisie par le conjoint survivant."
                            }
                            "QD PP"
                        }
                        div { class: "col-span-2 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_qd_pp().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_qd_pp().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-1 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_qd_pp().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-3 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800 grid grid-cols-3 items-stretch",
                            Output { signal: result.option_qd_pp().cumul_enfant() }
                            Output { signal: result.option_qd_pp().cumul_etat() }
                            Output { signal: result.option_qd_pp().cumul_notaire() }
                        }
                    }
                }
            }
            Rapport { snapshot, result, show_report }
        }
    }
}
