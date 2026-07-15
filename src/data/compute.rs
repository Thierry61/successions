use std::cmp;

use crate::data::{ABATTEMENT_AV, ABATTEMENT_DROITS, BeneficiaireState, FORFAIT_FRAIS_FUNERAIRES, HeritierState, REMISE_RP_FISCALE};
use crate::data::{InputState, OptionState, ResultState, FractionnementPropriete};

// Calcul au niveau des structures sous-jacentes (par opposition au wrapper de type store)
// - distribution des AV aux bénéficiaires et calcul des récompenses associées
// - liquiditation du la communauté
// - calcul de la succession et de la part du conjoint survivant hors succession
// - répartition de la succession pour chacune des 4 options possibles
pub fn compute(input: InputState, result: &mut ResultState) {
    // Si le conjoint survivant possède des AV alors il les conserve mais il doit une récompense à la communauté.
    // (au civil uniquement)
    let av = if input.ordre_deces {
        // Le survivant est votre conjoint
        input.av_conjoint_conjoint + input.av_conjoint_enfants
    } else {
        // Vous êtes le survivant
        input.av_vous_conjoint + input.av_vous_enfants
    };
    result.premier_deces_civil.recompense_due_par_le_survivant = av;
    result.premier_deces_civil.solde_recompenses += av;
    // Ces AV seront transmises aux enfants au 2eme décès avec des prélèvements fiscaux
    calcul_beneficiaire(&mut result.deuxieme_av_enfant, av / input.nb_enfants);

    // Si le défunt avait une AV au bénéfice des enfants alors ceux-ci reçoivent le capital mais le défunt doit une récompense à la communauté,
    // sauf si le survivant formalise une dispense de récompense avec le notaire.
    // (au civil et au fiscal)
    let av = if input.ordre_deces {
        // Vous êtes le défunt
        input.av_vous_enfants
    } else {
        // Votre conjoint est le défunt
        input.av_conjoint_enfants
    };
    if !input.dispense_recompense {
        result.premier_deces_civil.recompense_due_par_le_defunt = av;
        result.premier_deces_civil.solde_recompenses += av;
        result.premier_deces_fiscal.recompense_due_par_le_defunt = av;
        result.premier_deces_fiscal.solde_recompenses += av;
    }
    // Cette AV est transmise aux enfants au 1er décès avec des prélèvements fiscaux
    calcul_beneficiaire(&mut result.premier_av_enfant, av / input.nb_enfants);

    // Si le défunt avait une AV au bénéfice du conjoint alors celui-ci reçoit le capital sans qu'une récompense soit due.
    let av = if input.ordre_deces {
        // Vous êtes le défunt
        input.av_vous_conjoint
    } else {
        // Votre conjoint est le défunt
        input.av_conjoint_conjoint
    };
    // Cette AV est transmise au survivant sans prélèvements fiscaux
    result.premier_av_survivant.brut = av;
    result.premier_av_survivant.net = av;

    // Répartition des totaux des AV transmises aux bénéficiaires
    repartition_beneficiaire_total(&mut result.premier_av_total, &result.premier_av_enfant, input.nb_enfants, Some(&result.premier_av_survivant));
    repartition_beneficiaire_total(&mut result.deuxieme_av_total, &result.deuxieme_av_enfant, input.nb_enfants, None);

    // Actif brut de communauté : RP + placements hors AV/PER + biens meublants si le forfait mobilier n'est pas utilisé.
    // (au civil et au fiscal, avec un remise de 20% sur la RP au fiscal)
    result.premier_deces_civil.actif_brut_communaute = input.residence_principale + input.placements;
    result.premier_deces_fiscal.actif_brut_communaute = (input.residence_principale as f64 * (1.0 - REMISE_RP_FISCALE)) as i32 + input.placements;
    if !input.forfait_mobilier {
        result.premier_deces_civil.actif_brut_communaute += input.biens_meublants;
        result.premier_deces_fiscal.actif_brut_communaute += input.biens_meublants;
    }

    // Actif brut de communauté : actif net de communauté - dettes
    // (au civil et au fiscal)
    result.premier_deces_civil.actif_net_communaute = result.premier_deces_civil.actif_brut_communaute - input.dettes;
    result.premier_deces_fiscal.actif_net_communaute = result.premier_deces_fiscal.actif_brut_communaute - input.dettes;

    // Soldes de récompenses : récompenses dues à la communauté (on ne gère pas de récompenses dues par la communauté)
    // (au civil et au fiscal)
    result.premier_deces_civil.solde_recompenses = result.premier_deces_civil.recompense_due_par_le_survivant
        + result.premier_deces_civil.recompense_due_par_le_defunt;
    result.premier_deces_fiscal.solde_recompenses = result.premier_deces_fiscal.recompense_due_par_le_survivant
        + result.premier_deces_fiscal.recompense_due_par_le_defunt;

    // Actif net de communauté ajusté : actif net de communauté + solde de récompenses.
    // (au civil et au fiscal)
    result.premier_deces_civil.actif_net_communaute_ajuste = result.premier_deces_civil.actif_net_communaute
        + result.premier_deces_civil.solde_recompenses;
    result.premier_deces_fiscal.actif_net_communaute_ajuste = result.premier_deces_fiscal.actif_net_communaute
        + result.premier_deces_fiscal.solde_recompenses;

    // Actif brut successoral : actif net de communauté ajusté / 2 + biens propres du défunt (récompense due à la communauté à son passif)
    // (au civil et au fiscal)
    result.premier_deces_civil.actif_brut_succession =  result.premier_deces_civil.actif_net_communaute_ajuste / 2
        - result.premier_deces_civil.recompense_due_par_le_defunt;
    result.premier_deces_fiscal.actif_brut_succession =  result.premier_deces_fiscal.actif_net_communaute_ajuste / 2
        - result.premier_deces_fiscal.recompense_due_par_le_defunt;

    // Actif net successoral (= succession) : Actif brut successoral + biens meublants - frais funéraires
    // - au civil : frais funéraires réels, pas de biens meublants (soit déjà compté dans l'actif de communauté, soit répartition amiable)
    // - au fiscal : forfait de 1500 € pour les frais funéraires, biens meublants de 5% de l'actif brut de communauté si forfait mobilier utilisé
    result.premier_deces_civil.actif_net_succession = result.premier_deces_civil.actif_brut_succession - input.frais_funeraires;
    result.premier_deces_fiscal.actif_net_succession = result.premier_deces_fiscal.actif_brut_succession - FORFAIT_FRAIS_FUNERAIRES;
    if input.forfait_mobilier {
        // input.biens_meublants contient le forfait calculé dans l'UI
        result.premier_deces_fiscal.actif_net_succession += input.biens_meublants;
    }

    // Un actif net fiscal négatif est remis à 0 (pour éviter des impôts négatifs)
    result.premier_deces_fiscal.actif_net_succession = cmp::max(0, result.premier_deces_fiscal.actif_net_succession);

    // Part du survivant hors succession : actif net de communauté ajusté / 2 + biens propres du survivant (récompense due à la communauté à son passif)
    // (au civil uniquement)
    result.premier_deces_civil.part_survivant_hors_succession = result.premier_deces_civil.actif_net_communaute_ajuste / 2
        - result.premier_deces_civil.recompense_due_par_le_survivant;

    // On fait une copie de cette structure car la fonction calcul_option va en modifier un sous-ensemble
    // mais a besoin de lire le reste et elle ne peut pas l'emprunter à la fois en lecture et en écriture.
    let photo_result = result.clone();
    // Calcul des 4 options possibles que peut choisir le survivant.
    calcul_option(&mut result.option_totalite_us, FractionnementPropriete::new_totalite_us(), &input, &photo_result);
    calcul_option(&mut result.option_1_4_pp, FractionnementPropriete::new_1_4_pp(), &input, &photo_result);
    calcul_option(&mut result.option_1_4_pp_3_4_us, FractionnementPropriete::new_1_4_pp_3_4_us(), &input, &photo_result);
    calcul_option(&mut result.option_qd_pp, FractionnementPropriete::new_qd_pp(input.nb_enfants), &input, &photo_result);

    // Calcul des cumuls (pour éviter de créer des use_memo dans l'UI)
    result.option_totalite_us.cumul(input.nb_enfants);
    result.option_1_4_pp.cumul(input.nb_enfants);
    result.option_1_4_pp_3_4_us.cumul(input.nb_enfants);
    result.option_qd_pp.cumul(input.nb_enfants);
}

// Calcul d'une option
// TODO: faire les calculs dans le domaine f64 et à la fin seulement stocker les résultats dans des i32
fn calcul_option(option: &mut OptionState, fractionnement: FractionnementPropriete, input: &InputState, result: &ResultState) {
        
    // Calcul du 1er décès
    // -------------------

    option.premier_total.part_civile = result.premier_deces_civil.actif_net_succession;
    option.premier_total.part_fiscale = result.premier_deces_fiscal.actif_net_succession;

    // Fractionnement de l'actif net civil en US/NP/PP pour le survivant et les enfants
    // Deux combinaisons ne sont pas possibles : le survivant ne reçoit pas de NP et les enfants ne reçoivent pas d'US
    let age_survivant = if input.ordre_deces { input.age_conjoint } else { input.age_vous };
    let (us, np) = bareme_usufruit(age_survivant);
    let fractionnement_np_enfants = fractionnement.us_survivant;
    let fractionnement_pp_enfants = 1.0 - fractionnement.pp_survivant - fractionnement.us_survivant;
    option.premier_survivant.heritage_pp = (fractionnement.pp_survivant * result.premier_deces_civil.actif_net_succession as f64) as i32;
    option.premier_enfant.heritage_pp = (fractionnement_pp_enfants * result.premier_deces_civil.actif_net_succession as f64 / input.nb_enfants as f64) as i32;
    option.premier_enfant.heritage_np = (fractionnement_np_enfants * np * result.premier_deces_civil.actif_net_succession as f64 / input.nb_enfants as f64) as i32;
    option.premier_survivant.heritage_us = (fractionnement.us_survivant * us * result.premier_deces_civil.actif_net_succession as f64) as i32;

    // Totaux US/NP/PP
    option.premier_total.heritage_pp = option.premier_survivant.heritage_pp + input.nb_enfants * option.premier_enfant.heritage_pp;
    option.premier_total.heritage_np = input.nb_enfants * option.premier_enfant.heritage_np;
    option.premier_total.heritage_us = option.premier_survivant.heritage_us;

    // Part civile du survivant et des enfants
    option.premier_survivant.part_civile = option.premier_survivant.heritage_pp + option.premier_survivant.heritage_us;
    option.premier_enfant.part_civile = option.premier_enfant.heritage_pp + option.premier_enfant.heritage_np;

    // Coefficient permettant de calculer les part fiscales de chacun (survivant et enfants) à partir des parts civiles.
    // Il est mis à 0 si l'actif net civil est négatif ou nul (pour éviter des impôts négatifs ou une division par 0) 
    let coef = if result.premier_deces_civil.actif_net_succession > 0 {
        result.premier_deces_fiscal.actif_net_succession as f64 / result.premier_deces_civil.actif_net_succession as f64
    } else {
        0.0
    };
    option.premier_survivant.part_fiscale = (coef * option.premier_survivant.part_civile as f64) as i32;
    option.premier_enfant.part_fiscale = (coef * option.premier_enfant.part_civile as f64) as i32;

    // Calcul des droits et de l'héritage net des enfants
    let assiette_partage_total = if input.ignorer_couts_partage {
        0
    } else {
        cmp::max(0, option.premier_total.heritage_pp)   
    };
    calcul_heritier(&mut option.premier_enfant, Some(input.donations_partages/2/input.nb_enfants), result.premier_av_enfant.net, None, assiette_partage_total);

    // Calcul de l'héritage du survivant
    calcul_heritier(&mut option.premier_survivant, None, result.premier_av_survivant.net, None, assiette_partage_total);

    // Répartition des totaux des héritages
    repartition_heritier_total(&mut option.premier_total, &option.premier_enfant, input.nb_enfants, Some(&option.premier_survivant));
    option.premier_etat = option.premier_total.droits_succession + option.premier_total.droits_partage + result.premier_av_total.prelevement;
    option.premier_notaire = option.premier_total.emoluments_partage;

    // Calcul du 2eme décès
    // --------------------

    // Extinction d'usufruit (US + NP enfin reçus par les enfants et déjà taxés au 1er décès)
    option.deuxieme_total.extinction_us = option.premier_survivant.heritage_us + input.nb_enfants * option.premier_enfant.heritage_np;

    // Part civile:
    // La part du survivant hors succession + sa part en PP dans la succession + le capital de l'AV qu'il avait reçu du conjoint
    option.deuxieme_total.part_civile = result.premier_deces_civil.part_survivant_hors_succession + option.premier_survivant.heritage_pp + result.premier_av_survivant.net;

    // Part fiscale:
    // Un actif net fiscal négatif est remis à 0 (pour éviter des impôts négatifs)
    option.deuxieme_total.part_fiscale = cmp::max(0, option.deuxieme_total.part_civile);

    // Chaque enfant a sa part proportionnelle de ces éléments
    option.deuxieme_enfant.extinction_us = option.deuxieme_total.extinction_us / input.nb_enfants;
    option.deuxieme_enfant.part_civile = option.deuxieme_total.part_civile / input.nb_enfants;
    option.deuxieme_enfant.part_fiscale = option.deuxieme_total.part_fiscale / input.nb_enfants;

    // Calcul des droits et de l'héritage net des enfants
    let assiette_partage_total = if input.nb_enfants > 1 && !input.ignorer_couts_partage {
        cmp::max(0, option.deuxieme_total.extinction_us + option.deuxieme_total.part_civile)
    } else {
        0
    };
    calcul_heritier(&mut option.deuxieme_enfant, Some(input.donations_partages/2/input.nb_enfants), result.deuxieme_av_enfant.net, Some(input.nb_enfants), assiette_partage_total);

    // Répartition des totaux des héritages
    repartition_heritier_total(&mut option.deuxieme_total, &option.deuxieme_enfant, input.nb_enfants, None);
    option.deuxieme_etat = option.deuxieme_total.droits_succession + option.deuxieme_total.droits_partage + result.deuxieme_av_total.prelevement;
    option.deuxieme_notaire = option.deuxieme_total.emoluments_partage;
}

// Barème fiscal de l'usufruit et de la nue-propriété (cf. https://www.service-public.gouv.fr/particuliers/vosdroits/F934)
// en fonction de l'âge révolu (c'est-à-dire l'âge au dernier anniversaire)
fn bareme_usufruit(age_usufrutier: i32) -> (f64, f64) {
    let us = match age_usufrutier {
        a if a < 21 => 0.9,
        a if a < 31 => 0.8,
        a if a < 41 => 0.7,
        a if a < 51 => 0.6,
        a if a < 61 => 0.5,
        a if a < 71 => 0.4,
        a if a < 81 => 0.3,
        a if a < 91 => 0.2,
        _ => 0.1,
    };
    let np = 1.0 - us;
    (us, np)
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

// Calcul de l'héritage net des enfants ou du survivant, soit au 1er décès, soit au second décès.
// - la présence d'une donation partage (valant éventuellement 0) indique que l'on est en train de traiter les enfants (pris en compte dans l'abattement fiscal)
//   Rappel: les donations-partages traitées sont conjonctives, égalitaires et de moins de 15 ans.
// - la présence du nombre d'enfants indique que l'on est en train de traiter le 2eme décès (pour déterminer l'assiette des droits de partage)
// - l'assiette globale pour les émoluments de partage est passée en paramètre car ils ne sont pas proportionnels mais sont calculés
//   avec des pourcentages différents par tranche de l'actif brut et chaque héritier a une connaissance partielle de cette assiette.
// Nota: Les parts civiles et fiscales ainsi que les parties PP, NP et US ont déjà été calculées en amont
// en fonction de l'option choisie par le conjoint survivant.
fn calcul_heritier(heritier: &mut HeritierState, donations_partages: Option<i32>, av_net: i32, nb_enfants: Option<i32>, assiette_partage_globale: i32) {
    // Droits de succession, seuls les enfants les payent.
    // Pour le survivant le champ droits_succession n'est pas rempli et garde donc sa valeur par défaut 0.
    let mut enfant = false;
    if let Some(donations_partages) = donations_partages {
        enfant = true;
        // Les enfants sont susceptibles de payer des droits de succession en fonction de l'abattement et des donations reçues il y a moins de quinze ans.
        heritier.abattement = cmp::max(0, cmp::min(heritier.part_fiscale, ABATTEMENT_DROITS - donations_partages));
        heritier.taxable = heritier.part_fiscale - heritier.abattement;
        heritier.droits_succession = droits_en_ligne_direct(heritier.taxable as f64) as i32;
    }

    // Droits et émouluments de partage dont l'assiette dépend du décès.
    // Les liquidités pourraient être exclues du partage mais je n'ai pas implémenté cette possibilité,
    // le résultat est donc un majorant des droits susceptibles d'être prélevés.
    let assiette_partage_individuelle = if let Some(nb_enfants) = nb_enfants {
        // Au 2ème décès: l'assiette porte sur l'extinction d'usufruit et la part civile s'il y a plusieurs enfants.
        // Il n'y a pas de partage, donc pas de droits à payer s'il n'y a qu'un enfant.
        if nb_enfants > 1 {
            cmp::max(0, heritier.extinction_us + heritier.part_civile) as f64
        } else {
            0.0
        }
    } else {
        // Au 1er décès: l'assiette porte sur l'indivision c'est-à-dire la PP de l'héritier
        cmp::max(0, heritier.heritage_pp) as f64
    };
    if assiette_partage_globale == 0 {
        // Si l'assiette global est nulle alors il n'y a pas de droits ni d'émolument et cela évite une division par 0.
        heritier.droits_partage = 0;
        heritier.emoluments_partage = 0;
    } else {
        // Droits de partage proportionnels de 2.5%
        heritier.droits_partage = (0.025 * assiette_partage_individuelle) as i32;
        // Emoluments de partage non-proportionnels. Il faut les calculer globalement et les répartir au prorata de l'assiette individuelle.
        heritier.emoluments_partage = (partage_succession(assiette_partage_globale as f64) * assiette_partage_individuelle / assiette_partage_globale as f64) as i32;
    }

    // Calcul de l'héritage net
    heritier.heritage_net = heritier.part_civile - heritier.droits_succession - heritier.droits_partage - heritier.emoluments_partage;

    // Flux financier réel: il faut soustraire la nue-propriété et l'usufruit qui sont pour l'instant virtuels.
    // Le flux ne sera matérialisé qu'au 2eme décès lors de l'extinction de l'usufruit.
    // En l'absence de PP ou d'AV reçues le flux peut même être négatif au 1er décès pour les enfants
    // car ceux-ci payent des droits de succession sur des NP dont ils ne disposent pas encore.
    if enfant {
        // Au 2ème décès l'enfant touche l'extinction d'usufruit sans droits de succession (ils ont été payés au 1er décès)
        heritier.heritage_net += heritier.extinction_us;
        // Au 1er décès l'enfant ne touche pas réellement la nue propriété car elle est démembrée.
        heritier.flux_financier = heritier.heritage_net - heritier.heritage_np;
    } else {
        // Au 1er décès le survivant ne touche pas réellement l'usufruit car il est démembré.
        heritier.flux_financier = heritier.heritage_net - heritier.heritage_us;
    }

    // Dans tous les cas l'héritier touche le capital de l'AV dont il est bénéficiare
    heritier.flux_financier_avec_av = heritier.flux_financier + av_net;
}

// Répartition des totaux des héritages
fn repartition_heritier_total(total: &mut HeritierState, enfant: &HeritierState, nb_enfants: i32, survivant: Option<&HeritierState>) {
    total.abattement = enfant.abattement * nb_enfants;
    total.taxable = enfant.taxable * nb_enfants;
    total.droits_succession = enfant.droits_succession * nb_enfants;
    total.droits_partage = enfant.droits_partage * nb_enfants;
    total.emoluments_partage = enfant.emoluments_partage * nb_enfants;
    total.heritage_net = enfant.heritage_net * nb_enfants;
    total.flux_financier = enfant.flux_financier * nb_enfants;
    total.flux_financier_avec_av = enfant.flux_financier_avec_av * nb_enfants;
    if let Some(survivant) = survivant {
        total.abattement += survivant.abattement;
        total.taxable += survivant.taxable;
        total.droits_succession += survivant.droits_succession;
        total.droits_partage += survivant.droits_partage;
        total.emoluments_partage += survivant.emoluments_partage;
        total.heritage_net += survivant.heritage_net;
        total.flux_financier += survivant.flux_financier;
        total.flux_financier_avec_av += survivant.flux_financier_avec_av;
    }
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

// Calcul des capitaux décès nets pour une AV reçu par un enfant bénéficiaire
fn calcul_beneficiaire(beneficiaire: &mut BeneficiaireState, brut: i32) {
    beneficiaire.brut = brut;
    beneficiaire.abattement = cmp::min(brut, ABATTEMENT_AV);
    beneficiaire.taxable = brut - beneficiaire.abattement;
    beneficiaire.prelevement = prelevements_assurance_vie(beneficiaire.taxable as f64) as i32;
    beneficiaire.net = beneficiaire.brut - beneficiaire.prelevement;
}

// Répartition totale des AV transmises aux bénéficiaires
fn repartition_beneficiaire_total(total: &mut BeneficiaireState, enfant: &BeneficiaireState, nb_enfants: i32, survivant: Option<&BeneficiaireState>) {
    total.brut = enfant.brut * nb_enfants;
    total.abattement = enfant.abattement * nb_enfants;
    total.taxable = enfant.taxable * nb_enfants;
    total.prelevement = enfant.prelevement * nb_enfants;
    total.net = enfant.net * nb_enfants;
    if let Some(survivant) = survivant {
         total.brut += survivant.brut;
         total.abattement += survivant.abattement;
         total.taxable += survivant.taxable;
         total.prelevement += survivant.prelevement;
         total.net += survivant.net;
    }
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
fn test_bareme_usufruit() {
    assert_eq!(bareme_usufruit(61), (0.4, 0.6));
    assert_eq!(bareme_usufruit(66), (0.4, 0.6));
    assert_eq!(bareme_usufruit(70), (0.4, 0.6));
    assert_eq!(bareme_usufruit(71), (0.3, 0.7));
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

    // Vérification que si l'assiette est nulle alors les émoluments sont nuls
    assert_eq!(to_cents(partage_succession(0.0)), 0);
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
