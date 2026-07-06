use std::cmp;

use crate::data::{ABATTEMENT_AV, BeneficiaireState, FORFAIT_FRAIS_FUNERAIRES, REMISE_RP_FISCALE};
use crate::data::{InputState, ResultState, FractionnementPropriete};

// Calcul au niveau des structures sous-jacentes
pub fn compute(input: InputState, result: &mut ResultState) {
    // Si le conjoint survivant a des AV alors il les conserve mais il doit une récompense à la communauté.
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
    // Ces AV seront transmises aux enfants au 2eme décès
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
    // Cette AV est transmise aux enfants
    calcul_beneficiaire(&mut result.premier_av_enfant, av / input.nb_enfants);

    // Si le défunt avait une AV au bénéfice du conjoint alors celui-ci reçoit le capital sans qu'une récompense soit due.
    let av = if input.ordre_deces {
        // Vous êtes le défunt
        input.av_vous_conjoint
    } else {
        // Votre conjoint est le défunt
        input.av_conjoint_conjoint
    };
    // Cette AV est transmise au survivant
    result.premier_av_survivant.brut = av;
    result.premier_av_survivant.net = av;

    // Cumul des AV transmises aux bénéficiaires
    calcul_beneficiaire_total(&mut result.premier_av_total, &result.premier_av_enfant, input.nb_enfants, Some(&result.premier_av_survivant));
    calcul_beneficiaire_total(&mut result.deuxieme_av_total, &result.deuxieme_av_enfant, input.nb_enfants, None);

    // Actif brut de communauté : RP + placements hors AV/PER + biens meublants si le forfait mobilier n'est pas utilisé.
    // (au civil et au fiscal avec un remise de 20% sur la RP au fiscal)
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
    // - au civil : frais funéraires réels, pas de biens meublants (soit déjà compté dans l'actif de communauté, soit répartion amiable)
    // - au fiscal : forfait de 1500 € pour les frais funéraires, biens meublants de 5% de l'actif brut de communauté si forfait mobilier utilisé
    result.premier_deces_civil.actif_net_succession = result.premier_deces_civil.actif_brut_succession - input.frais_funeraires;
    result.premier_deces_fiscal.actif_net_succession = result.premier_deces_fiscal.actif_brut_succession - FORFAIT_FRAIS_FUNERAIRES;
    if input.forfait_mobilier {
        result.premier_deces_fiscal.actif_net_succession += input.biens_meublants;
    }

    // Part du survivant hors succession : actif net de communauté ajusté / 2 + biens propres du survivant (récompense due à la communauté à son passif)
    // (au civil uniquement)
    result.premier_deces_civil.part_survivant_hors_succession = result.premier_deces_civil.actif_net_communaute_ajuste / 2
        - result.premier_deces_civil.recompense_due_par_le_survivant;

    // Calcul des cumuls (pour éviter de créer des use_memo dans l'UI)
    result.option_totalite_us.cumul();
    result.option_1_4_pp.cumul();
    result.option_1_4_pp_3_4_us.cumul();
    result.option_qd_pp.cumul();
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

fn calcul_beneficiaire(beneficiaire: &mut BeneficiaireState, brut: i32) {
    beneficiaire.brut = brut;
    beneficiaire.abattement = cmp::min(brut, ABATTEMENT_AV);
    beneficiaire.taxable = brut - beneficiaire.abattement;
    beneficiaire.prelevement = prelevements_assurance_vie(beneficiaire.taxable as f64) as i32;
    beneficiaire.net = beneficiaire.brut - beneficiaire.prelevement;
}

fn calcul_beneficiaire_total(total: &mut BeneficiaireState, enfant: &BeneficiaireState, nb_enfants: i32, survivant: Option<&BeneficiaireState>) {
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
