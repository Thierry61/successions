use dioxus::prelude::*;

mod compute;
use crate::data::compute::compute;

pub const FORFAIT_FRAIS_FUNERAIRES: i32 = 1500;
pub const REMISE_RP_FISCALE: f64 = 0.2;
pub const DEFAUT_NB_ENFANTS: i32 = 2;
pub const ABATTEMENT_AV: i32 = 152_500;
pub const ABATTEMENT_DROITS: i32 = 100_000;
pub const ABATTEMENT_PER: i32 = 30_500;

// Crée un cookie ou le détruit si la valeur est la valeur par défaut de l'entrée
// (plus exactement ajoute dans la variable js l'instruction javascript effectuant cette action)
fn set_cookie(js: &mut String, name: &'static str, val: i32, default_val: i32) {
    // 400 days is the upper limit for Max-Age attribute (see https://developer.chrome.com/blog/cookie-max-age-expires)
    let max_age = 400 * 24 * 3600;
    if val != default_val {
        // Crée le cookie. Utilise l'attribut expires plutôt que maxAge car la disponibilité de ce dernier est plus limitée.
        js.push_str(&format!(r#"cookieStore.set({{name: "{name}", value: {val}, expires: Date.now() + {max_age}}}), "#));
    } else {
        // Détruit le cookie
        js.push_str(&format!(r#"cookieStore.delete("{name}"), "#));
    }
}

#[test]
fn test_set_cookie() {
    let max_age = 400 * 24 * 3600;
    assert_eq!(max_age, 34560000);
    let mut js = String::new();
    set_cookie(&mut js, "nb_enfants", 3, 2);
    set_cookie(&mut js, "dettes", 0, 0);
    let expected = format!(
        r#"cookieStore.set({{name: "nb_enfants", value: 3, expires: Date.now() + 34560000}}), cookieStore.delete("dettes"), "#
    );
    assert_eq!(&js, &expected)
}

// Calcul des biens meublants lorsque le forfait mobilier est utilisé
// (5% de l'actif successoral brut)
pub fn calcul_biens_meublants(residence_principale: i32, placements: i32, dettes: i32) -> i32 {
    ((0.05
        * (residence_principale as f64 * (1.0 - REMISE_RP_FISCALE) + placements as f64
            - dettes as f64))
        / 2.0) as i32
}

#[test]
fn test_calcul_biens_meublants() {
    assert_eq!(calcul_biens_meublants(200_000, 180_000, 0), 8_500);
    assert_eq!(calcul_biens_meublants(200_000, 180_000, 20_000), 8_000);
}

#[derive(Store, Default)]
pub struct InputState {
    nb_enfants: i32,
    dettes: i32,
    residence_principale: i32,
    placements: i32,
    biens_meublants: i32,
    frais_funeraires: i32,
    donations_partages: i32,
    forfait_mobilier: bool,
    ordre_deces: bool,
    deces_survivant_apres_70_ans: bool,
    dispense_recompense: bool,
    ignorer_couts_partage: bool,
    ignorer_declaration_succession: bool,
    age_vous: i32,
    age_conjoint: i32,
    av_vous_conjoint: i32,
    av_conjoint_conjoint: i32,
    av_vous_enfants: i32,
    av_conjoint_enfants: i32,
    per_vous_conjoint: i32,
    per_conjoint_conjoint: i32,
}
impl InputState {
    // Génère une structure avec les valeurs par défaut
    pub fn new() -> Self {
        Self {
            nb_enfants: DEFAUT_NB_ENFANTS,
            ordre_deces: true,
            deces_survivant_apres_70_ans: true,
            dispense_recompense: true,
            ..Default::default()
        }
    }
    // Génère une structure avec les valeurs par défaut surchargées par la valeur des cookies
    pub fn new_from_cookies(cookies: &str) -> Self {
        let mut ret = Self::new();
        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            let vec: Vec<_> = cookie.split('=').collect();
            if vec.len() != 2 {
                continue;
            }
            let name = vec[0].trim();
            let val = vec[1].trim();
            match name {
                "nb_enfants" => {
                    if let Ok(val) = val.parse() {
                        ret.nb_enfants = val;
                    }
                }
                "dettes" => {
                    if let Ok(val) = val.parse() {
                        ret.dettes = val;
                    }
                }
                "residence_principale" => {
                    if let Ok(val) = val.parse() {
                        ret.residence_principale = val;
                    }
                }
                "placements" => {
                    if let Ok(val) = val.parse() {
                        ret.placements = val;
                    }
                }
                "biens_meublants" => {
                    if let Ok(val) = val.parse() {
                        ret.biens_meublants = val;
                    }
                }
                "frais_funeraires" => {
                    if let Ok(val) = val.parse() {
                        ret.frais_funeraires = val;
                    }
                }
                "donations_partages" => {
                    if let Ok(val) = val.parse() {
                        ret.donations_partages = val;
                    }
                }
                "forfait_mobilier" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.forfait_mobilier = val == 1;
                    }
                }
                "ordre_deces" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.ordre_deces = val == 1;
                    }
                }
                "deces_survivant_apres_70_ans" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.deces_survivant_apres_70_ans = val == 1;
                    }
                }
                "dispense_recompense" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.dispense_recompense = val == 1;
                    }
                }
                "ignorer_couts_partage" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.ignorer_couts_partage = val == 1;
                    }
                }
                "ignorer_declaration_succession" => {
                    if let Ok(val) = val.parse::<i32>() {
                        ret.ignorer_declaration_succession = val == 1;
                    }
                }
                "age_vous" => {
                    if let Ok(val) = val.parse() {
                        ret.age_vous = val;
                    }
                }
                "age_conjoint" => {
                    if let Ok(val) = val.parse() {
                        ret.age_conjoint = val;
                    }
                }
                "av_vous_conjoint" => {
                    if let Ok(val) = val.parse() {
                        ret.av_vous_conjoint = val;
                    }
                }
                "av_conjoint_conjoint" => {
                    if let Ok(val) = val.parse() {
                        ret.av_conjoint_conjoint = val;
                    }
                }
                "av_vous_enfants" => {
                    if let Ok(val) = val.parse() {
                        ret.av_vous_enfants = val;
                    }
                }
                "av_conjoint_enfants" => {
                    if let Ok(val) = val.parse() {
                        ret.av_conjoint_enfants = val;
                    }
                }
                "per_vous_conjoint" => {
                    if let Ok(val) = val.parse() {
                        ret.per_vous_conjoint = val;
                    }
                }
                "per_conjoint_conjoint" => {
                    if let Ok(val) = val.parse() {
                        ret.per_conjoint_conjoint = val;
                    }
                }
                _ => continue,
            }
            // On recalcule les biens meublants si le forfait mobilier est utilisé
            // (dès fois que le cookie biens meublants soit erroné)
            if ret.forfait_mobilier {
                ret.biens_meublants =
                    calcul_biens_meublants(ret.residence_principale, ret.placements, ret.dettes);
            }
            // Idem pour le cas où le cookie relatif au deces avant 70 ans est erroné
            let age_survivant = if ret.ordre_deces {
                ret.age_conjoint
            } else {
                ret.age_vous
            };
            if age_survivant >= 70 {
                ret.deces_survivant_apres_70_ans = true;
            }
        }
        ret
    }
    // J'ai codé en dur cette fonction car je n'ai pas trouvé de moyen de reconstruire automatiquement la structure sous-jacente au store
    pub fn from(store: Store<InputState>) -> Self {
        Self {
            nb_enfants: *store.nb_enfants().read(),
            dettes: *store.dettes().read(),
            residence_principale: *store.residence_principale().read(),
            placements: *store.placements().read(),
            biens_meublants: *store.biens_meublants().read(),
            frais_funeraires: *store.frais_funeraires().read(),
            donations_partages: *store.donations_partages().read(),
            forfait_mobilier: *store.forfait_mobilier().read(),
            ordre_deces: *store.ordre_deces().read(),
            deces_survivant_apres_70_ans: *store.deces_survivant_apres_70_ans().read(),
            dispense_recompense: *store.dispense_recompense().read(),
            ignorer_couts_partage: *store.ignorer_couts_partage().read(),
            ignorer_declaration_succession: *store.ignorer_declaration_succession().read(),
            age_vous: *store.age_vous().read(),
            age_conjoint: *store.age_conjoint().read(),
            av_vous_conjoint: *store.av_vous_conjoint().read(),
            av_conjoint_conjoint: *store.av_conjoint_conjoint().read(),
            av_vous_enfants: *store.av_vous_enfants().read(),
            av_conjoint_enfants: *store.av_conjoint_enfants().read(),
            per_vous_conjoint: *store.per_vous_conjoint().read(),
            per_conjoint_conjoint: *store.per_conjoint_conjoint().read(),
        }
    }
    // Idem pour cette fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<InputState>) {
        store.nb_enfants().set(self.nb_enfants);
        store.dettes().set(self.dettes);
        store.residence_principale().set(self.residence_principale);
        store.placements().set(self.placements);
        store.biens_meublants().set(self.biens_meublants);
        store.frais_funeraires().set(self.frais_funeraires);
        store.donations_partages().set(self.donations_partages);
        store.forfait_mobilier().set(self.forfait_mobilier);
        store.ordre_deces().set(self.ordre_deces);
        store
            .deces_survivant_apres_70_ans()
            .set(self.deces_survivant_apres_70_ans);
        store.dispense_recompense().set(self.dispense_recompense);
        store
            .ignorer_couts_partage()
            .set(self.ignorer_couts_partage);
        store
            .ignorer_declaration_succession()
            .set(self.ignorer_declaration_succession);
        store.age_vous().set(self.age_vous);
        store.age_conjoint().set(self.age_conjoint);
        store.av_vous_conjoint().set(self.av_vous_conjoint);
        store.av_conjoint_conjoint().set(self.av_conjoint_conjoint);
        store.av_vous_enfants().set(self.av_vous_enfants);
        store.av_conjoint_enfants().set(self.av_conjoint_enfants);
        store.per_vous_conjoint().set(self.per_vous_conjoint);
        store
            .per_conjoint_conjoint()
            .set(self.per_conjoint_conjoint);
    }
    // Idem pour cette fonction codée en dur générant une chaine permettant de sauvergarder les entrées dans des cookies
    pub fn to_cookies(store: Store<InputState>) -> String {
        let def = InputState::new();
        let mut js = "await Promise.all([".to_string();
        set_cookie(
            &mut js,
            "nb_enfants",
            *store.nb_enfants().read(),
            def.nb_enfants,
        );
        set_cookie(&mut js, "dettes", *store.dettes().read(), def.dettes);
        set_cookie(
            &mut js,
            "residence_principale",
            *store.residence_principale().read(),
            def.residence_principale,
        );
        set_cookie(
            &mut js,
            "placements",
            *store.placements().read(),
            def.placements,
        );
        set_cookie(
            &mut js,
            "biens_meublants",
            *store.biens_meublants().read(),
            def.biens_meublants,
        );
        set_cookie(
            &mut js,
            "frais_funeraires",
            *store.frais_funeraires().read(),
            def.frais_funeraires,
        );
        set_cookie(
            &mut js,
            "donations_partages",
            *store.donations_partages().read(),
            def.donations_partages,
        );
        set_cookie(
            &mut js,
            "forfait_mobilier",
            if *store.forfait_mobilier().read() {
                1
            } else {
                0
            },
            if def.forfait_mobilier { 1 } else { 0 },
        );
        set_cookie(
            &mut js,
            "ordre_deces",
            if *store.ordre_deces().read() { 1 } else { 0 },
            if def.ordre_deces { 1 } else { 0 },
        );
        set_cookie(
            &mut js,
            "deces_survivant_apres_70_ans",
            if *store.deces_survivant_apres_70_ans().read() {
                1
            } else {
                0
            },
            if def.deces_survivant_apres_70_ans {
                1
            } else {
                0
            },
        );
        set_cookie(
            &mut js,
            "dispense_recompense",
            if *store.dispense_recompense().read() {
                1
            } else {
                0
            },
            if def.dispense_recompense { 1 } else { 0 },
        );
        set_cookie(
            &mut js,
            "ignorer_couts_partage",
            if *store.ignorer_couts_partage().read() {
                1
            } else {
                0
            },
            if def.ignorer_couts_partage { 1 } else { 0 },
        );
        set_cookie(
            &mut js,
            "ignorer_declaration_succession",
            if *store.ignorer_declaration_succession().read() {
                1
            } else {
                0
            },
            if def.ignorer_declaration_succession {
                1
            } else {
                0
            },
        );
        set_cookie(&mut js, "age_vous", *store.age_vous().read(), def.age_vous);
        set_cookie(
            &mut js,
            "age_conjoint",
            *store.age_conjoint().read(),
            def.age_conjoint,
        );
        set_cookie(
            &mut js,
            "av_vous_conjoint",
            *store.av_vous_conjoint().read(),
            def.av_vous_conjoint,
        );
        set_cookie(
            &mut js,
            "av_conjoint_conjoint",
            *store.av_conjoint_conjoint().read(),
            def.av_conjoint_conjoint,
        );
        set_cookie(
            &mut js,
            "av_vous_enfants",
            *store.av_vous_enfants().read(),
            def.av_vous_enfants,
        );
        set_cookie(
            &mut js,
            "av_conjoint_enfants",
            *store.av_conjoint_enfants().read(),
            def.av_conjoint_enfants,
        );
        set_cookie(
            &mut js,
            "per_vous_conjoint",
            *store.per_vous_conjoint().read(),
            def.per_vous_conjoint,
        );
        set_cookie(
            &mut js,
            "per_conjoint_conjoint",
            *store.per_conjoint_conjoint().read(),
            def.per_conjoint_conjoint,
        );
        js.push_str("]);");
        js
    }
}

#[test]
fn test_new_from_cookies() {
    let cookies = "nb_enfants=3; dettes=180; forfait_mobilier=0; dispense_recompense=1";
    let input = InputState::new_from_cookies(cookies);
    assert_eq!(input.residence_principale, 0);
    assert_eq!(input.nb_enfants, 3);
    assert_eq!(input.dettes, 180);
    assert_eq!(input.forfait_mobilier, false);
    assert_eq!(input.dispense_recompense, true);
}

// Bénéficiaire d'une assurance-vie
#[derive(Store, Default, Clone)]
pub struct BeneficiaireState {
    brut: i32,
    abattement: i32,
    taxable: i32,
    prelevement: i32,
    net: i32,
}
impl BeneficiaireState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<BeneficiaireState>) {
        store.brut().set(self.brut);
        store.abattement().set(self.abattement);
        store.taxable().set(self.taxable);
        store.prelevement().set(self.prelevement);
        store.net().set(self.net);
    }
}

// Heritier dans une succession. Attention : Ne pas regrouper avec la notion de bénéficiaire
// car le premier dépend de l'option choisie, mais pas le second.
#[derive(Store, Default, Clone)]
pub struct HeritierState {
    // Champs pour le 1er décès
    heritage_pp: i32,
    heritage_np: i32,
    heritage_us: i32,
    // Champ pour le 2ème décès
    extinction_us: i32,
    // Champs communs aux 2 décès
    part_civile: i32,
    part_fiscale: i32,
    abattement: i32,
    taxable: i32,
    droits_succession: i32,
    droits_partage: i32,
    emoluments_partage: i32,
    emoluments_declaration_succession: i32,
    heritage_net: i32,
    flux_financier: i32,
    flux_financier_avec_av: i32,
}
impl HeritierState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<HeritierState>) {
        store.heritage_pp().set(self.heritage_pp);
        store.heritage_np().set(self.heritage_np);
        store.heritage_us().set(self.heritage_us);
        store.extinction_us().set(self.extinction_us);
        store.part_civile().set(self.part_civile);
        store.part_fiscale().set(self.part_fiscale);
        store.abattement().set(self.abattement);
        store.taxable().set(self.taxable);
        store.droits_succession().set(self.droits_succession);
        store.droits_partage().set(self.droits_partage);
        store.emoluments_partage().set(self.emoluments_partage);
        store
            .emoluments_declaration_succession()
            .set(self.emoluments_declaration_succession);
        store.heritage_net().set(self.heritage_net);
        store.flux_financier().set(self.flux_financier);
        store
            .flux_financier_avec_av()
            .set(self.flux_financier_avec_av);
    }
}

// Résultat du calcul pour une option choisie par le conjoint survivant
// ("totalite en US", "1/4 en PP", "1/4_PP et 3/4 en US" ou "quotité disponible en PP")
#[derive(Store, Default, Clone)]
pub struct OptionState {
    // Données du premier décès
    premier_survivant: HeritierState,
    premier_enfant: HeritierState,
    premier_total: HeritierState,
    premier_etat: i32,
    premier_notaire: i32,
    // Données du deuxième décès
    deuxieme_us_survivant: i32,
    deuxieme_pp_survivant: i32,
    deuxieme_actif_net_succession_civil: i32,
    deuxieme_actif_net_succession_fiscal: i32,
    deuxieme_enfant: HeritierState,
    deuxieme_total: HeritierState,
    deuxieme_etat: i32,
    deuxieme_notaire: i32,
    cumul_enfant: i32,
    cumul_etat: i32,
    cumul_notaire: i32,
    cumul_total: i32,
}
impl OptionState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<OptionState>) {
        self.premier_survivant.to(store.premier_survivant().into());
        self.premier_enfant.to(store.premier_enfant().into());
        self.premier_total.to(store.premier_total().into());
        store.premier_etat().set(self.premier_etat);
        store.premier_notaire().set(self.premier_notaire);
        store
            .deuxieme_us_survivant()
            .set(self.deuxieme_us_survivant);
        store
            .deuxieme_pp_survivant()
            .set(self.deuxieme_pp_survivant);
        store
            .deuxieme_actif_net_succession_civil()
            .set(self.deuxieme_actif_net_succession_civil);
        store
            .deuxieme_actif_net_succession_fiscal()
            .set(self.deuxieme_actif_net_succession_fiscal);
        self.deuxieme_enfant.to(store.deuxieme_enfant().into());
        self.deuxieme_total.to(store.deuxieme_total().into());
        store.deuxieme_etat().set(self.deuxieme_etat);
        store.deuxieme_notaire().set(self.deuxieme_notaire);
        store.cumul_enfant().set(self.cumul_enfant);
        store.cumul_etat().set(self.cumul_etat);
        store.cumul_notaire().set(self.cumul_notaire);
        store.cumul_total().set(self.cumul_total);
    }
    // Calcul des cumuls (pour éviter de créer des use_memo dans l'UI)
    pub fn cumul(&mut self, nb_enfants: i32) {
        self.cumul_enfant = self.premier_enfant.flux_financier_avec_av
            + self.deuxieme_enfant.flux_financier_avec_av;
        self.cumul_etat = self.premier_etat + self.deuxieme_etat;
        self.cumul_notaire = self.premier_notaire + self.deuxieme_notaire;
        self.cumul_total = self.cumul_enfant * nb_enfants;
    }
}

// Gestion des fractions PP/US/NP en fonction du choix du conjoint survivants.
// Toutes les fractions sont des nombres entre 0 et 1.
pub struct FractionnementPropriete {
    // Fraction en PP du survivant
    pp_survivant: f64,
    // Fraction en US du survivant
    us_survivant: f64,
    // Nota:
    // - Fraction en NP des enfants = US_survivant
    // - Fraction en PP des enfants = 1 - fraction_PP_survivant - fraction_US_survivant
}

impl FractionnementPropriete {
    // Création du fractionnement pour chaque option
    fn new_totalite_us() -> Self {
        Self {
            pp_survivant: 0.0,
            us_survivant: 1.0,
        }
    }
    fn new_1_4_pp() -> Self {
        Self {
            pp_survivant: 1.0 / 4.0,
            us_survivant: 0.0,
        }
    }
    fn new_1_4_pp_3_4_us() -> Self {
        Self {
            pp_survivant: 1.0 / 4.0,
            us_survivant: 3.0 / 4.0,
        }
    }
    fn new_qd_pp(nb_enfants: i32) -> Self {
        let qd = match nb_enfants {
            0 => 3.0 / 4.0,
            1 => 1.0 / 2.0,
            2 => 1.0 / 3.0,
            _ => 1.0 / 4.0,
        };
        Self {
            pp_survivant: qd,
            us_survivant: 0.0,
        }
    }
}

#[derive(Store, Default, Clone)]
pub struct PremierDeces {
    recompense_due_par_le_survivant: i32,
    recompense_due_par_le_defunt: i32,
    actif_brut_communaute: i32,
    solde_recompenses: i32,
    actif_net_communaute: i32,
    actif_net_communaute_ajuste: i32,
    actif_brut_succession: i32,
    actif_net_succession: i32,
    part_survivant_hors_succession: i32,
}
impl PremierDeces {
    pub fn to(&self, store: Store<PremierDeces>) {
        store
            .recompense_due_par_le_survivant()
            .set(self.recompense_due_par_le_survivant);
        store
            .recompense_due_par_le_defunt()
            .set(self.recompense_due_par_le_defunt);
        store
            .actif_brut_communaute()
            .set(self.actif_brut_communaute);
        store.solde_recompenses().set(self.solde_recompenses);
        store.actif_net_communaute().set(self.actif_net_communaute);
        store
            .actif_net_communaute_ajuste()
            .set(self.actif_net_communaute_ajuste);
        store
            .actif_brut_succession()
            .set(self.actif_brut_succession);
        store.actif_net_succession().set(self.actif_net_succession);
        store
            .part_survivant_hors_succession()
            .set(self.part_survivant_hors_succession);
    }
}

#[derive(Store, Default, Clone)]
pub struct ResultState {
    premier_deces_civil: PremierDeces,
    premier_deces_fiscal: PremierDeces,
    // Capital de l'AV reçu par le conjoint survivant
    premier_av_survivant: BeneficiaireState,
    // Capital de l'AV reçu par chaque enfant au 1er décès
    premier_av_enfant: BeneficiaireState,
    // Somme des AV reçues par tous les bénéficiaires au 1er décès
    premier_av_total: BeneficiaireState,
    // Capital du PER reçu par le conjoint survivant au 1er décès (exonéré)
    premier_per: i32,
    premier_per_total: i32,
    // Capital de l'AV reçu par chaque enfant au 2ème décès
    deuxieme_av_enfant: BeneficiaireState,
    // Somme des AV reçues par tous les bénéficiaires au 2ème décès décès
    deuxieme_av_total: BeneficiaireState,
    // Capital du PER reçu par chaque enfant au 2ème décès (soumis aux droits de succession après abattement de 30 500 €)
    deuxieme_per: i32,
    deuxieme_per_total: i32,
    option_totalite_us: OptionState,
    option_1_4_pp: OptionState,
    option_1_4_pp_3_4_us: OptionState,
    option_qd_pp: OptionState,
}
impl ResultState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<ResultState>) {
        self.premier_deces_civil
            .to(store.premier_deces_civil().into());
        self.premier_deces_fiscal
            .to(store.premier_deces_fiscal().into());
        self.premier_av_survivant
            .to(store.premier_av_survivant().into());
        self.premier_av_enfant.to(store.premier_av_enfant().into());
        self.premier_av_total.to(store.premier_av_total().into());
        store.premier_per().set(self.premier_per);
        store.premier_per_total().set(self.premier_per_total);
        self.deuxieme_av_enfant
            .to(store.deuxieme_av_enfant().into());
        self.deuxieme_av_total.to(store.deuxieme_av_total().into());
        store.deuxieme_per().set(self.deuxieme_per);
        store.deuxieme_per_total().set(self.deuxieme_per_total);
        self.option_totalite_us
            .to(store.option_totalite_us().into());
        self.option_1_4_pp.to(store.option_1_4_pp().into());
        self.option_1_4_pp_3_4_us
            .to(store.option_1_4_pp_3_4_us().into());
        self.option_qd_pp.to(store.option_qd_pp().into());
    }
    // Wrapper du calcul au niveaux des store
    pub fn store_compute(
        store_input: Store<InputState>,
        snapshot_input: Store<InputState>,
        store_result: Store<ResultState>,
    ) {
        // Copie figée des inputs (pour que le rapport ne soit pas modifié après génération)
        let snapshot = InputState::from(store_input);
        snapshot.to(snapshot_input);

        // Initialisation des résultats à 0
        let mut result = ResultState::default();

        // Récupération des données sous-jacentes au store des entrées
        let input = InputState::from(store_input);

        // Calcul des résultats
        compute(input, &mut result);

        // Surcharge du store des résultats
        result.to(store_result);
    }
}
