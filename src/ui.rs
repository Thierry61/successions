use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, X};

use crate::data::history::History;
use crate::data::{
    calcul_biens_meublants, CheckStateStoreExt, HeritierStateStoreExt, InputState,
    InputStateStoreExt, OptionStateStoreExt, ResultState, ResultStateStoreExt, DEFAUT_NB_ENFANTS,
};
use crate::report::{format_num, Rapport};

// Croix en rouge
#[component]
fn RedX() -> Element {
    rsx! {
        div { class: "pt-1 pl-3 text-red-600 dark:text-red-400",
            X { class: "size-5", stroke_width: 3 }
        }
    }
}

// Check mark en vert
#[component]
fn GreenCheck() -> Element {
    rsx! {
        div { class: "pt-1 pl-3 text-green-600 dark:text-green-400",
            Check { class: "size-5", stroke_width: 3 }
        }
    }
}

// Vérification du total distribué dans une option
#[component]
fn CheckOption(show_report: ReadSignal<bool>, is_ok: ReadSignal<bool>) -> Element {
    rsx! {
        if !*show_report.read() {
            div {}
        } else if *is_ok.read() {
            GreenCheck {}
        } else {
            RedX {}
        }
    }
}

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
                        span { class: "hidden sm:inline", " {optional}" }
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
    tooltip: ReadSignal<String>,
    signal: WriteSignal<bool>,
    disabled: Option<Signal<bool>>,
) -> Element {
    rsx! {
        div { class: "tooltip-top tooltip",
            span { class: "tooltip-text ml-12!", {tooltip} }
            input {
                id,
                class: "mx-2 my-1 accent-blue-50 dark:accent-blue-700",
                r#type: "checkbox",
                onclick: move |_| {
                    let new_val = !*signal.read();
                    // Ecrit le signal et ajoute une entrée dans la pile des undo
                    use_context::<Store<History>>().write().add_bool(id, signal, new_val);
                },
                checked: signal,
                disabled,
            }
            label { r#for: id, "{lab}" }
        }
    }
}

// Gestion d'un champ input avec ou sans label.
// Un mémo permet le formatage de la valeur affichée :
// - En mode saisie le champ a un backgroud rosé et la valeur n'est pas formatée.
// - En mode hors saisie il a le background normal du thème et la valeur est formatée
//   avec des blancs comme séparateurs de milliers.
// Le basculement du mode est réalisé à l'aide de l'état is_focused.
// Nota:
// - Une solution baséee sur un overlay marcherait aussi.
// - Dans les 2 solutions le Ctrl-Z inter-champs ne marche qu'après un hot reload.
//   Une gestion spécifique des Undo/Redo a donc été codée.
#[component]
fn Input(
    id: &'static str,
    signal: WriteSignal<i32>,
    is_nb_enfants: bool,
    disabled: Option<Signal<bool>>,
) -> Element {
    // Affichage formaté avec des blancs comme séparateurs de milliers quand l'élement n'est pas sélectionné
    let mut is_focused = use_signal(|| false);
    let mut force_refresh = use_signal(|| 0);
    let signal_str = use_memo(move || {
        let _ = *force_refresh.read();
        if *is_focused.read() {
            (*signal.read()).to_string()
        } else {
            format_num(*signal.read())
        }
    });
    // Mémorise la valeur d'entrée de champ. C'est nécessaire car le signal est mis à jour
    // dans le oninput pour rafraichir les dépendances à chaque caractère frappé.
    let mut old_val = use_signal(|| 0);
    // Récupère l'historique des undo/redo
    let mut history = use_context::<Store<History>>();
    // Traitement des événements oninput et onchange.
    let mut manage_input_and_change = move |e: Event<FormData>, is_change: bool| {
        // Récupère la valeur saisie
        let new_val = e.value();
        if !is_change {
            if !e.valid() {
                e.prevent_default();
                return;
            }
            // Stocke la nouvelle valeur saisie sauf en mode input si le premier chiffre à gauche vaut 0
            // pour que l'utilisateur se soit pas supris de voir disparaitre une série de 0 à gauche
            // alors qu'il voulait juste remplacer le chiffre le plus signicatif (par exemple il voulait
            // remplacer 30000 par 40000, en effacant le 3 et en tapant 4 à la place).
            if new_val.starts_with('0') {
                return;
            }
        }
        // Valeur par défaut
        let def_val = if is_nb_enfants {
            DEFAUT_NB_ENFANTS
        } else {
            i32::default()
        };
        // En mode change un champ vide est remplacé par la valeur par défaut
        // (en mode input le champ étant invalide on est sorti plus haut)
        let ajusted_val = if new_val.is_empty() {
            def_val
        } else {
            // Si la nouvelle valeur est invalide ou négative alors le unwrap_or met :
            // - la valeur courante en mode input (si l'utilise frappe un caractère erroné
            //   alors il ne faut pas que tout le contenu disparaisse pour qu'il puisse effacer ce caractère)
            // - la valeur par défaut en mode change
            new_val
                .parse::<u32>()
                .unwrap_or_else(|_| if is_change { def_val } else { signal() } as u32)
                as i32
        };
        if is_change {
            // Ecrit le signal et ajoute une entrée dans la pile des undo
            history
                .write()
                .add_i32(id, signal, *old_val.read(), ajusted_val);
            // Force le rafraichissement du champ formaté (c'est nécessaire parfois)
            *force_refresh.write() += 1;
        } else {
            // Ecrit le signal mais n'enregistre rien dans la pile des undo. Ces écritures
            // intermédiaires dans le oniput permettent de rafraichir les dépendances
            // en direct au cours de la saisie.
            *signal.write() = ajusted_val;
        }
    };
    rsx! {
        div { class: "relative inline-block h-5 m-1",
            input {
                id,
                class: "w-17 pr-1 text-end bg-blue-50 dark:bg-blue-500 rounded-sm",
                class: "disabled:bg-gray-300 dark:disabled:bg-gray-500",
                class: if *is_focused.read() { "bg-pink-100 dark:bg-pink-600" },
                class: "remove-arrow",
                r#type: "text",
                min: if is_nb_enfants { "1" } else { "0" },
                pattern: "[0-9]+",
                disabled,
                onfocus: move |_| {
                    *old_val.write() = *signal.read();
                    is_focused.set(true);
                },
                onblur: move |_| {
                    is_focused.set(false);
                },
                // Vérifie le champ caractère par caractère
                oninput: move |e: Event<FormData>| {
                    manage_input_and_change(e, false);
                },
                // Vérifie le champ à la fin de la saisie
                onchange: move |e: Event<FormData>| {
                    manage_input_and_change(e, true);
                },
                value: signal_str,
            }
        }
    }
}

#[component]
fn Output(signal: ReadSignal<i32>) -> Element {
    let num = format_num(*signal.read());
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
    is_nb_enfants: Option<bool>,
) -> Element {
    rsx! {
        div {
            div {
                class: "w-48 px-2 py-1 flex flex-row justify-between bg-blue-100 dark:bg-blue-600 rounded-lg drop-shadow-md",
                class: "border border-blue-300 dark:border-blue-800",
                div { class: if !tooltip.is_empty() { "tooltip-top tooltip" },
                    span { class: "tooltip-text", {tooltip} }
                    {lab}
                }
                Input {
                    id,
                    signal,
                    is_nb_enfants: is_nb_enfants == Some(true),
                }
            }
        }
    }
}

#[component]
fn InputWithoutLabel(id: &'static str, signal: WriteSignal<i32>) -> Element {
    rsx! {
        div {
            Input { id, signal, is_nb_enfants: false }
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
    // Indique si l'option deces_survivant_apres_70_ans est désactivée
    let mut deces_survivant_apres_70_ans_disabled = use_signal(|| false);
    // Tooltip du forfait mobilier fiscal
    let mut tooltip_forfait_mobilier = use_signal(String::new);
    // Gére les dépendances inter-champs
    use_effect(move || {
        // Calcul du forfait mobilier fiscal (formaté et affiché dans le tooltip)
        let residence_principale = *input.residence_principale().read();
        let placements = *input.placements().read();
        let dettes = *input.dettes().read();
        let forfait_mobilier = calcul_biens_meublants(residence_principale, placements, dettes);
        let forfait_mobilier = format_num(forfait_mobilier);
        tooltip_forfait_mobilier.set(format!("Forfait de 5% de l'actif successoral brut pour les biens meublants sur le plan fiscal (soit {forfait_mobilier} €)."));
        // Si le conjoint survivant (d'après l'ordre des décès) est déjà agé de plus de 70 ans
        // alors le flag deces_survivant_apres_70_ans est positionné à true et est rendu non modifiable
        let age_survivant = if *input.ordre_deces().read() {
            *input.age_conjoint().read()
        } else {
            *input.age_vous().read()
        };
        if age_survivant >= 70 {
            input.deces_survivant_apres_70_ans().set(true);
            deces_survivant_apres_70_ans_disabled.set(true);
        } else {
            deces_survivant_apres_70_ans_disabled.set(false);
        }
    });
    // Historique des Undo/Redo
    let mut history = use_store(History::new);
    // Context provider pour éviter de passer l'historique en paramètre à chaque composant
    use_context_provider(|| history);

    rsx! {
        // Décommenter la ligne suivante pour debugger les cookies
        // "Cookies: {cookies}"
        // Une forme est nécessaire pour déclencher le calcul en entrant un retour-chariot sur n'importe quel champ.
        form {
            class: "text-sm",
            // TODO: même si cet évenement est géré au niveau de la forme il faut que le focus soit sur un input
            onkeydown: move |e: Event<KeyboardData>| {
                if e.modifiers().ctrl() {
                    match e.key() {
                        Key::Character(ref s) if s == "z" => {
                            e.prevent_default();
                            history.write().undo();
                        }
                        Key::Character(ref s) if s == "y" => {
                            e.prevent_default();
                            history.write().redo();
                        }
                        _ => {}
                    }
                }
            },
            div { class: "m-3",
                details { open: "false",
                    summary { class: "mt-2 leading-6 font-semibold select-none",
                        "Hypothèses principales (cliquer pour développer)"
                    }
                    ul { class: "ml-5 list-disc list-outside",
                        li {
                            "Le couple est marié sous le régime légal (communauté réduite aux acquêts) et a au moins un enfant."
                        }
                        li {
                            "Tous les éléments sont communs (enfants, biens, dettes et fonds ayant alimenté les placements et donations)."
                        }
                        li { "Les versements sur les assurances-vie ont été effectués avant 70 ans." }
                        li {
                            "Les bénéficiaires des assurances-vie sont soit les enfants, soit le conjoint puis les enfants."
                        }
                        li {
                            "Les PER modélisés sont des PER assurantiels et leurs bénéficiaires sont le conjoint puis les enfants."
                        }
                    }
                }
            }
            div { id: "inputs", class: "m-2 flex flex-wrap gap-4",
                InputWithLabel {
                    id: "nb-enfants",
                    lab: "Nombre d'enfants",
                    tooltip: "Nombre d'enfants communs du couple, doit être supérieur ou égal à 1.",
                    signal: input.nb_enfants(),
                    is_nb_enfants: true,
                }
                InputWithLabel {
                    id: "RP",
                    lab: "Résidence principale",
                    tooltip: "Pour abattement de 20% dans le calcul des droits (plan fiscal).",
                    signal: input.residence_principale(),
                }
                InputWithLabel {
                    id: "placements",
                    lab: "Placements hors AV/PER",
                    tooltip: "Placements sauf AV et PER qui ont une fiscalité spécifique et une éventuelle récompense à prendre en compte.",
                    signal: input.placements(),
                }
                InputWithLabel {
                    id: "dettes",
                    lab: "Dettes et impôts",
                    tooltip: "Dettes de la communauté, y compris les impôts restants à payer.",
                    signal: input.dettes(),
                }
                InputWithLabel {
                    id: "biens-meublants",
                    lab: "Biens meublants",
                    tooltip: "Intégrés dans l'actif de communauté uniquement sur le plan civil si forfait mobilier ou sur les 2 plans (fiscal et civil) sinon",
                    signal: input.biens_meublants(),
                }
                InputWithLabel {
                    id: "frais-funeraires",
                    lab: "Frais funéraires réels",
                    tooltip: "Frais funéraire réels déduits de l'actif successoral net (plan civil), par opposition au forfait de 1500€ déduit sur le plan fiscal.",
                    signal: input.frais_funeraires(),
                }
                InputWithLabel {
                    id: "donations-partages",
                    lab: "Donations partages",
                    tooltip: "Donations-partages de moins de 15 ans, conjonctives, égalitaires et hors dons Sarkozy (plan fiscal).",
                    signal: input.donations_partages(),
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
                            span { class: "tooltip-text",
                                "Détermine le barème fiscal de l'usufruit et de la nue-propriété."
                            }
                            "Ages des époux"
                        }
                        InputWithoutLabel { id: "age_vous", signal: input.age_vous() }
                        InputWithoutLabel { id: "age_conjoint", signal: input.age_conjoint() }
                        div { class: "col-span-2 tooltip-top tooltip",
                            span { class: "tooltip-text w-65!",
                                "Les prélèvements sociaux sur les plus-values sont à déduire (fonds euros pour l'année courante et UC depuis l'origine)."
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
                            span { class: "tooltip-text w-65!",
                                "Les prélèvements sociaux sur les plus-values sont à déduire (fonds euros pour l'année courante et UC depuis l'origine)."
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
                            span { class: "tooltip-text w-50!",
                                "Les prélèvements sociaux sur les plus-values ne sont pas à déduire."
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
                            tooltip: tooltip_forfait_mobilier,
                            signal: input.forfait_mobilier(),
                        }
                        Checkbox {
                            id: "ordre-décès",
                            lab: "Ordre des décès : vous puis votre conjoint",
                            tooltip: "Si la case est cochée alors la simulation suppose que vous décédiez avant votre conjoint (le contraire sinon).",
                            signal: input.ordre_deces(),
                        }
                        Checkbox {
                            id: "deces-survivant-apres-70-ans",
                            lab: "Décès de l'époux survivant après 70 ans.",
                            tooltip: "Détermine la fiscalité du PER du conjoint survivant à son décès.",
                            signal: input.deces_survivant_apres_70_ans(),
                            disabled: deces_survivant_apres_70_ans_disabled,
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
                // Avant l'ajout de la colonne Vérif il y avait 7 colonnes décomposées en 1 + 2 + 1 + 3.
                // Cette nouvelle colonne est moitié plus petite que les autres et a donc une taille
                // approximative de 1/7/2. Tout a été multiplié par 7 et après tatonnement la meilleure
                // décomposition semble être : 49 = 7 + 13 + 7 + 22.
                Fieldset { legend: "Résultats", optional: "", center: false,
                    div {
                        id: "résultats",
                        class: "sm:px-2 px-0 pb-2 grid grid-cols-49 gap-x-0 sm:gap-x-2 gap-y-0",
                        div { class: "col-span-7 mt-3",
                            button {
                                class: "px-4 py-2 font-bold bg-green-100 text-green-700 dark:bg-green-600 dark:text-white",
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
                        div { class: "col-span-13",
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
                        div { class: "col-span-7",
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
                        div { class: "col-span-22",
                            Fieldset {
                                legend: "Cumul des 2 décès",
                                optional: "",
                                center: true,
                                div { class: "pl-2 grid grid-cols-7 items-end",
                                    div { class: "tooltip tooltip-top col-span-2",
                                        span { class: "tooltip-text w-65!",
                                            "Valeur reçue en pleine-propriété par chaque enfant, incluant les assurances-vie dont il est bénéficiaire."
                                        }
                                        "Chaque"
                                        br {}
                                        "enfant"
                                    }
                                    div { class: "pl-1 tooltip tooltip-top col-span-2",
                                        span { class: "tooltip-text", "Impôts perçus par l'Etat." }
                                        "Etat"
                                    }
                                    div { class: "pl-2 tooltip tooltip-top col-span-2",
                                        span { class: "tooltip-text w-35!",
                                            "Emoluments perçus par le notaire."
                                        }
                                        "Notaire"
                                    }
                                    div { class: "pl-2 tooltip tooltip-top",
                                        span { class: "tooltip-text w-45!",
                                            "Vérification que le total distribué aux enfants, à l'Etat et au notaire est égal à l'actif net de départ."
                                        }
                                        "Vérif."
                                        br {}
                                        "total"
                                    }
                                }
                            }
                        }
                        div { class: "col-span-7 ml-1 tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option totalité en usufruit choisie par le conjoint survivant."
                            }
                            "100% US"
                        }
                        div { class: "col-span-13 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_totalite_us().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_totalite_us().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-7 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_totalite_us().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-22 border-x border-blue-300 dark:border-blue-800 grid grid-cols-7 items-stretch",
                            div { class: "col-span-2",
                                Output { signal: result.option_totalite_us().cumul_enfant() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_totalite_us().cumul_etat() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_totalite_us().cumul_notaire() }
                            }
                            CheckOption {
                                show_report,
                                is_ok: result.check().option_totalite_us(),
                            }
                        }
                        div { class: "col-span-7 ml-1 tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option 1/4 en pleine propriété choisie par le conjoint survivant."
                            }
                            "¼ PP"
                        }
                        div { class: "col-span-13 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_1_4_pp().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_1_4_pp().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-7 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_1_4_pp().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-22 border-x border-blue-300 dark:border-blue-800 grid grid-cols-7 items-stretch",
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp().cumul_enfant() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp().cumul_etat() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp().cumul_notaire() }
                            }
                            CheckOption {
                                show_report,
                                is_ok: result.check().option_1_4_pp(),
                            }
                        }
                        div { class: "col-span-7 ml-1 tooltip-right tooltip",
                            span { class: "tooltip-text",
                                "Option 1/4 en pleine propriété et 3/4 en usufruit choisie par le conjoint survivant."
                            }
                            "¼ PP ¾ US"
                        }
                        div { class: "col-span-13 border-x border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_1_4_pp_3_4_us().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_1_4_pp_3_4_us().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-7 border-x border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_1_4_pp_3_4_us().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-22 border-x border-blue-300 dark:border-blue-800 grid grid-cols-7 items-stretch",
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp_3_4_us().cumul_enfant() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp_3_4_us().cumul_etat() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_1_4_pp_3_4_us().cumul_notaire() }
                            }
                            CheckOption {
                                show_report,
                                is_ok: result.check().option_1_4_pp_3_4_us(),
                            }
                        }
                        // Tooltip top au lieu de right pour éviter une bande blanche en bas
                        div { class: "col-span-7 ml-1 tooltip-right tooltip",
                            span { class: "tooltip-text w-50!",
                                "Option quotité disponible en pleine propriété choisie par le conjoint survivant."
                            }
                            "QD PP"
                        }
                        div { class: "col-span-13 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800 grid grid-cols-2 items-stretch",
                            Output { signal: result.option_qd_pp().premier_survivant().flux_financier_avec_av() }
                            Output { signal: result.option_qd_pp().premier_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-7 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800",
                            Output { signal: result.option_qd_pp().deuxieme_enfant().flux_financier_avec_av() }
                        }
                        div { class: "col-span-22 border-b border-x rounded-b-lg border-blue-300 dark:border-blue-800 grid grid-cols-7 items-stretch",
                            div { class: "col-span-2",
                                Output { signal: result.option_qd_pp().cumul_enfant() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_qd_pp().cumul_etat() }
                            }
                            div { class: "col-span-2",
                                Output { signal: result.option_qd_pp().cumul_notaire() }
                            }
                            CheckOption {
                                show_report,
                                is_ok: result.check().option_qd_pp(),
                            }
                        }
                    }
                }
            }
            Rapport { snapshot, result, show_report }
        }
    }
}
