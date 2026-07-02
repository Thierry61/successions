use dioxus::prelude::*;

pub const DEFAUT_NB_ENFANTS: i32 = 2;

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
    dispense_recompense: bool,
    ignorer_couts_partage: bool,
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
    pub fn new () -> Self {
        Self { nb_enfants: DEFAUT_NB_ENFANTS, forfait_mobilier: true, ordre_deces: true, dispense_recompense: true, ..Default::default() }
    }
    // J'ai codé en dur cette fonction car je n'ai pas trouvé de moyen de reconstruire automatiquement la structure sous-jacente au store
    pub fn from (store: Store<InputState>) -> Self {
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
            dispense_recompense: *store.dispense_recompense().read(),
            ignorer_couts_partage: *store.ignorer_couts_partage().read(),
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
        store.dispense_recompense().set(self.dispense_recompense);
        store.ignorer_couts_partage().set(self.ignorer_couts_partage);
        store.age_vous().set(self.age_vous);
        store.age_conjoint().set(self.age_conjoint);
        store.av_vous_conjoint().set(self.av_vous_conjoint);
        store.av_conjoint_conjoint().set(self.av_conjoint_conjoint);
        store.av_vous_enfants().set(self.av_vous_enfants);
        store.av_conjoint_enfants().set(self.av_conjoint_enfants);
        store.per_vous_conjoint().set(self.per_vous_conjoint);
        store.per_conjoint_conjoint().set(self.per_conjoint_conjoint);
    }
}

// Bénéficiaire d'une assurance-vie
#[derive(Store, Default)]
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
#[derive(Store, Default)]
pub struct HeritierState {
    // TODO: d'autres champs
    heritage_net: i32,
    flux_financier: i32,
    flux_financier_avec_av: i32,
}
impl HeritierState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<HeritierState>) {
        store.heritage_net().set(self.heritage_net);
        store.flux_financier().set(self.flux_financier);
        store.flux_financier_avec_av().set(self.flux_financier_avec_av);
    }
}

// Résultat du calcul pour une option choisie par le conjoint survivant
// ("totalite en US", "1/4 en PP", "1/4_PP et 3/4 en US" ou "quotité disponible en PP")
#[derive(Store, Default)]
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
}
impl OptionState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<OptionState>) {
        self.premier_survivant.to(store.premier_survivant().into());
        self.premier_enfant.to(store.premier_enfant().into());
        self.premier_total.to(store.premier_total().into());
        store.premier_etat().set(self.premier_etat);
        store.premier_notaire().set(self.premier_notaire);
        store.deuxieme_us_survivant().set(self.deuxieme_us_survivant);
        store.deuxieme_pp_survivant().set(self.deuxieme_pp_survivant);
        store.deuxieme_actif_net_succession_civil().set(self.deuxieme_actif_net_succession_civil);
        store.deuxieme_actif_net_succession_fiscal().set(self.deuxieme_actif_net_succession_fiscal);
        self.deuxieme_enfant.to(store.deuxieme_enfant().into());
        self.deuxieme_total.to(store.deuxieme_total().into());
        store.deuxieme_etat().set(self.deuxieme_etat);
        store.deuxieme_notaire().set(self.deuxieme_notaire);
        store.cumul_enfant().set(self.cumul_enfant);
        store.cumul_etat().set(self.cumul_etat);
        store.cumul_notaire().set(self.cumul_notaire);
    }
    // Calcul des cumuls (pour éviter de créer des use_memo dans l'UI)
    fn cumul(&mut self) {
        self.cumul_enfant = self.premier_enfant.flux_financier_avec_av + self.deuxieme_enfant.flux_financier_avec_av;
        self.cumul_etat = self.premier_etat + self.deuxieme_etat;
        self.cumul_notaire = self.premier_notaire + self.deuxieme_notaire;
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
        Self { pp_survivant: 0.0, us_survivant: 1.0 }
    }
    fn new_1_4_pp() -> Self {
        Self { pp_survivant: 1.0/4.0, us_survivant: 0.0 }
    }
    fn new_1_4_pp_3_4_us() -> Self {
        Self { pp_survivant: 1.0/4.0, us_survivant: 3.0/4.0 }
    }
    fn new_qd_pp(nb_enfants: i32) -> Self {
        let qd = match nb_enfants { 0 => 3.0/4.0, 1 => 1.0/2.0, 2 => 1.0/3.0, _ => 1.0/4.0 };
        Self { pp_survivant: qd, us_survivant: 0.0 }
    }
}

// Concernant les récompenses, voici le résumé des échanges sur le forum MoneyVox :
// - AV du conjoint survivant : le survivant conserve son contrat mais il doit une récompense à la communauté (sur le plan fiscal civil uniquement)
// - PERin du conjoint survivant : le survivant conserve son contrat sans devoir de récompense à la communauté (le contrat est un propre du survivant)
// - AV ou PERin du défunt au bénéfice du conjoint : pas de récompenses dues (le capital devient un propre du survivant)
// - AV du défunt au bénéfice des enfants : le défunt doit une récompense à la communauté (sauf si le survivant a proposé une dispense de récompense)
// Nota :
// - toutes les récompenses gérées sont dues à la communauté et sont donc inscrites à l'actif de la communauté
// - en parallèle elle sont inscrites au passif d'un propre (soit du survivant, soit du défunt)

#[derive(Store, Default)]
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
        store.recompense_due_par_le_survivant().set(self.recompense_due_par_le_survivant);
        store.recompense_due_par_le_defunt().set(self.recompense_due_par_le_defunt);
        store.actif_brut_communaute().set(self.actif_brut_communaute);
        store.solde_recompenses().set(self.solde_recompenses);
        store.actif_net_communaute().set(self.actif_net_communaute);
        store.actif_net_communaute_ajuste().set(self.actif_net_communaute_ajuste);
        store.actif_brut_succession().set(self.actif_brut_succession);
        store.actif_net_succession().set(self.actif_net_succession);
        store.part_survivant_hors_succession().set(self.part_survivant_hors_succession);
    }
}

#[derive(Store, Default)]
pub struct ResultState {
    premier_deces_civil: PremierDeces,
    premier_deces_fiscal: PremierDeces,
    // Valeur AV reçue par le conjoint survivant
    premier_av_survivant: BeneficiaireState,
    // Valeur AV reçue par chaque enfant au 1er décès
    premier_av_enfant: BeneficiaireState,
    premier_av_total: BeneficiaireState,
    // Contrats AV en propre détenus par le conjoint survivant
    deuxieme_av_survivant: i32,
    // Valeur AV reçue par chaque enfant au 2ème décès
    deuxieme_av_enfant: BeneficiaireState,
    deuxieme_av_total: BeneficiaireState,
    option_totalite_us: OptionState,
    option_1_4_pp: OptionState,
    option_1_4_pp_3_4_us: OptionState,
    option_qd_pp: OptionState,
}
impl ResultState {
    // Fonction codée en dur pour réinitialiser le store à partir de la structure sous-jacente
    pub fn to(&self, store: Store<ResultState>) {
        self.premier_deces_civil.to(store.premier_deces_civil().into());
        self.premier_deces_fiscal.to(store.premier_deces_fiscal().into());
        self.premier_av_survivant.to(store.premier_av_survivant().into());
        self.premier_av_enfant.to(store.premier_av_enfant().into());
        self.premier_av_total.to(store.premier_av_total().into());
        store.deuxieme_av_survivant().set(self.deuxieme_av_survivant);
        self.deuxieme_av_enfant.to(store.deuxieme_av_enfant().into());
        self.deuxieme_av_total.to(store.deuxieme_av_total().into());
        self.option_totalite_us.to(store.option_totalite_us().into());
        self.option_1_4_pp.to(store.option_1_4_pp().into());
        self.option_1_4_pp_3_4_us.to(store.option_1_4_pp_3_4_us().into());
        self.option_qd_pp.to(store.option_qd_pp().into());
    }
    // Wrapper du calcul au niveaux des store
    pub fn store_compute(store_input: Store<InputState>, snapshot_input: Store<InputState>, store_result: Store<ResultState>) {
        // Copie figée des inputs (pour que le rapport ne soit pas modifié après génération)
        let snapshot = InputState::from(store_input);
        snapshot.to(snapshot_input);

        // Initialisation des résultats à 0
        let mut result = ResultState::default();

        // Récupération des données sous-jacentes au store des entrées
        let input = InputState::from(store_input);

        // Calcul des résultats
        Self::compute(input, &mut result);

        // Surcharge du store des résultats
        result.to(store_result);
    }
    // Calcul au niveau des structures sous-jacentes
    fn compute(input: InputState, result: &mut ResultState) {
        // Traitement de test. TODO: faire le vrai calcul
        result.option_totalite_us.premier_enfant.flux_financier_avec_av = input.per_conjoint_conjoint;

        // Calcul des cumuls (pour éviter de créer des use_memo dans l'UI)
        result.option_totalite_us.cumul();
        result.option_1_4_pp.cumul();
        result.option_1_4_pp_3_4_us.cumul();
        result.option_qd_pp.cumul();
    }
}

// Droits de succession en ligne direct sur la part taxable après abattement de 100 000 €
fn droits_en_ligne_direct(part: f64) -> f64 {
    let res = if part<=8_072.0 {
            part*0.05
        } else {
            8_072.0*0.05 + if part<=12_109.0 {
                (part-8_072.0)*0.10
            } else {
                (12_109.0-8_072.0)*0.10 + if part<=15_932.0 {
                    (part-12_109.0)*0.15
                } else {
                    (15_932.0-12_109.0)*0.15 + if part<=552_324.0 {
                        (part-15_932.0)*0.20
                    } else {
                        (552_324.0-15_932.0)*0.20 + if part<=902_838.0 {
                            (part-552_324.0)*0.30
                        } else {
                            (902_838.0-552_324.0)*0.30 + if part<=1_805_677.0 {
                                (part-902_838.0)*0.40
                            } else {
                                (1_805_677.0-902_838.0)*0.40+(part-1_805_677.0)*0.45 } } } } } };
    // Arrondi au centime
    (res*100.0).round()/100.0
}

// Prélèvements sur une assurance-vie au dela de l'abattement de 152 500 €
// (cf. https://www.impots.gouv.fr/international-particulier/questions/je-suis-beneficiaire-dune-assurance-vie-comment-sont-imposees)
fn prelevements_assurance_vie(part: f64) -> f64 {
    let res = if part<=700_000.0 {
        part*0.2
    } else {
        700_000.0*0.2+(part-700_000.0)*0.3125
    };
    // Arrondi au centime
    (res*100.0).round()/100.0
}

// Emoluments du notaire (cf. https://www.service-public.gouv.fr/particuliers/vosdroits/F795
// et https://blog.qoridor.fr/article/emoluments-notaire-succession-bareme-2026)
fn partage_succession(part: f64) -> f64 {
    let res = if part<=6_500.0 {
        4.837*part
    } else {
        4.837*6_500.0 + if part<=17_000.0 {
            1.995*(part-6_500.0)
        } else {
            1.995*(17_000.0-6_500.0) + if part<=60_000.0 {
                1.33*(part-17_000.0)
            } else {
                1.33*(60_000.0-17_000.0)+0.998*(part-60_000.0)
            }
        }
    };
    // Division par 100 car les coefficients sont des pourcentages
    let res = res/100.0;
    // Ajout de la TVA à 20%
    let res = 1.2 * res;
    // Arrondi au centime
    (res*100.0).round()/100.0
}
fn declaration_succession(part: f64) -> f64 {
    let res = if part<=6_500.0 {
        1.548*part
    } else {
        1.548*6_500.0 + if part<=17_000.0 {
            0.851*(part-6_500.0)
        } else {
            0.851*(17_000.0-6_500.0) + if part<=30_000.0 {
                0.58*(part-17_000.0)
            } else {
                0.58*(30_000.0-17_000.0)+0.426*(part-30_000.0)
            }
        }
    };
    // Division par 100 car les coefficients sont des pourcentages
    let res = res/100.0;
    // Ajout de la TVA à 20%
    let res = 1.2 * res;
    // Arrondi au centime
    (res*100.0).round()/100.0
}

#[test]
fn test_fractionnement() {
    for t in [(1,2),(2,3),(3,4),(4,4)] {
        let fractionnement = FractionnementPropriete::new_qd_pp(t.0);
        assert_eq!(fractionnement.pp_survivant, 1.0/t.1 as f64);
        assert_eq!(fractionnement.us_survivant, 0.0);
    }
}

// Je ne suis pas sûr qu'il soit nécessaire de convertir les centimes en entiers
// mais c'est plus prudent pour comparer des floats. Une autre façon de faire serait
// de comparer la valeur absolue de la différence avec un epsilon mais cela impliquerait
// l'usage de assert! qui est moins bien que assert_eq! (en cas d'erreur des tests).
#[cfg(test)]
fn to_cents(x: f64) -> i64 {
    (x * 100.0).round() as i64
}

#[test]
fn test_droits_en_ligne_direct() {
    // Données du test provenant de https://www.service-public.gouv.fr/simulateur/calcul/droits-succession#main
    // (sauf les centimes que j'ai récupérés des tests eux-mêmes !)
    assert_eq!(to_cents(droits_en_ligne_direct(1_000.0)), 50_00);
    assert_eq!(to_cents(droits_en_ligne_direct(10_000.0)), 596_40);
    assert_eq!(to_cents(droits_en_ligne_direct(14_000.0)), 1_090_95);
    assert_eq!(to_cents(droits_en_ligne_direct(100_000.0)), 18_194_35);
    assert_eq!(to_cents(droits_en_ligne_direct(800_000.0)), 182_961_95);
    assert_eq!(to_cents(droits_en_ligne_direct(1_000_000.0)), 252_678_15);
    assert_eq!(to_cents(droits_en_ligne_direct(2_000_000.0)), 662_394_30);
}

#[test]
fn test_prelevements_assurance_vie () {
    assert_eq!(to_cents(prelevements_assurance_vie(47_500.0)), 9_500_00);
    assert_eq!(to_cents(prelevements_assurance_vie(700_000.0)), 140_000_00);
    assert_eq!(to_cents(prelevements_assurance_vie(747_500.0)), 154_843_75);
}

#[test]
fn test_emoluments_partage_succession() {
    // Valeurs dans chaque tranche
    assert_eq!(to_cents(partage_succession(3_000.0)), 174_13);
    assert_eq!(to_cents(partage_succession(10_000.0)), 461_08);
    assert_eq!(to_cents(partage_succession(30_000.0)), 836_14);
    assert_eq!(to_cents(partage_succession(100_000.0)), 1_793_98);

    // Bornes exactes
    assert_eq!(to_cents(partage_succession(6_500.0)), 377_29);
    assert_eq!(to_cents(partage_succession(17_000.0)), 628_66);
    assert_eq!(to_cents(partage_succession(60_000.0)), 1_314_94);
}

#[test]
fn test_emoluments_declaration_succession() {
    // Valeurs dans chaque tranche
    assert_eq!(to_cents(declaration_succession(3_000.0)), 55_73);
    assert_eq!(to_cents(declaration_succession(10_000.0)), 156_49);
    assert_eq!(to_cents(declaration_succession(20_000.0)), 248_85);
    assert_eq!(to_cents(declaration_succession(100_000.0)), 676_29);

    // Bornes exactes
    assert_eq!(to_cents(declaration_succession(6_500.0)), 120_74);
    assert_eq!(to_cents(declaration_succession(17_000.0)), 227_97);
    assert_eq!(to_cents(declaration_succession(30_000.0)), 318_45);
}
