// Tests d'intégration pour vérifier la compatibilité avec le simulateur de MACSF.
// Ils peuvent être exécutés avec la commande : cargo test --test macsf

mod common;
use common::CHECKED;
use successions::data::{compute::compute, InputState, ResultState, EPSILON};

// Pour retrouver les résultats MACSF il faut cocher la case
// "Abattement sur la résidence principale au 1er décès"
#[test]
fn test_residence_principale() {
    let input = InputState {
        nb_enfants: 2,
        residence_principale: 300_000,
        placements: 600_000,
        age_vous: 65,
        age_conjoint: 64,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert!(i32::abs(result.option_totalite_us.premier_total.droits_succession - 6_789) < EPSILON);
    assert!(i32::abs(result.option_totalite_us.cumul_etat - 53_178) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp.premier_total.droits_succession - 19_389) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp.cumul_etat - 88_278) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp_3_4_us.premier_total.droits_succession - 0) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat - 68_889) < EPSILON);

    assert!(i32::abs(result.option_qd_pp.premier_total.droits_succession - 12_389) < EPSILON);
    assert!(i32::abs(result.option_qd_pp.cumul_etat - 88_778) < EPSILON);

    assert_eq!(result.check, CHECKED);
}

// Pour retrouver les résultats sur MACSF il faut cocher la case "Forfait mobilier".
#[test]
fn test_forfait_mobilier() {
    // Le résultat est le même entre :
    // - un forfait mobilier de 15 000 au fiscal et des biens meublants de 30 000 au civil
    // - pas de forfait mobilier et des biens meublants de 30 000 à la fois au fiscal et au civil
    for (biens_meublants, forfait_mobilier) in [(30_000, true), (30_000, false)] {
        let input = InputState {
            nb_enfants: 2,
            placements: 600_000,
            biens_meublants,
            age_vous: 65,
            age_conjoint: 64,
            forfait_mobilier,
            ignorer_couts_partage: true,
            ignorer_declaration_succession: true,
            ..Default::default()
        };

        let mut result = ResultState::default();
        compute(&input, &mut result);

        assert_eq!(
            result.premier_deces_fiscal.forfait_mobilier,
            if input.forfait_mobilier { 15_000 } else { 0 }
        );

        assert_eq!(result.option_totalite_us.premier_total.droits_succession, 0);
        assert!(i32::abs(result.option_totalite_us.cumul_etat - 19_389) < EPSILON);

        assert!(i32::abs(result.option_1_4_pp.premier_total.droits_succession - 3_639) < EPSILON);
        assert!(i32::abs(result.option_1_4_pp.cumul_etat - 38_778) < EPSILON);

        assert_eq!(
            result.option_1_4_pp_3_4_us.premier_total.droits_succession,
            0
        );
        assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat - 35_139) < EPSILON);

        assert!(i32::abs(result.option_qd_pp.premier_total.droits_succession - 500) < EPSILON);
        assert!(i32::abs(result.option_qd_pp.cumul_etat - 40_889) < EPSILON);

        assert_eq!(result.check, CHECKED);
    }
}

// Pour retrouver les résultats sur MACSF il faut saisir 1 500 dans la case "Vous" du forfait funéraire.
// Du moins, ceci est vrai quand il n'y a pas d'US. En présence d'US, MACSF trouve une centaine d'euros
// en moins sur les droits au 2ème décès.
#[test]
fn test_frais_funeraires() {
    let input = InputState {
        nb_enfants: 2,
        placements: 900_000,
        frais_funeraires: 1_500,
        age_vous: 65,
        age_conjoint: 64,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert!(i32::abs(result.option_totalite_us.premier_total.droits_succession - 10_209) < EPSILON);
    // Différence inexpliquée de 118 €
    assert!(i32::abs(result.option_totalite_us.cumul_etat - 118 - 56_478) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp.premier_total.droits_succession - 23_664) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp.cumul_etat - 92_478) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp_3_4_us.premier_total.droits_succession - 91) < EPSILON);
    // Différence inexpliquée de 87 €
    assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat - 87 - 68_815) < EPSILON);

    assert!(i32::abs(result.option_qd_pp.premier_total.droits_succession - 16_189) < EPSILON);
    assert!(i32::abs(result.option_qd_pp.cumul_etat - 92_478) < EPSILON);

    assert_eq!(result.check, CHECKED);
}

// C'est un peu analogue : On retrouve les résultats sur MACSF sauf avec les options avec de l'US.
// La différence apparait encore sur le calcul des droits au 2ème décès.
#[test]
fn test_dettes() {
    let input = InputState {
        nb_enfants: 2,
        placements: 900_000,
        dettes: 100_000,
        age_vous: 65,
        age_conjoint: 64,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert!(i32::abs(result.option_totalite_us.premier_total.droits_succession - 4_389) < EPSILON);
    // Différence inexpliquée de 4 000 €
    assert!(i32::abs(result.option_totalite_us.cumul_etat - 4_000 - 36_778) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp.premier_total.droits_succession - 16_389) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp.cumul_etat - 72_778) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp_3_4_us.premier_total.droits_succession - 0) < EPSILON);
    // Différence inexpliquée de 3 000 €
    assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat - 3_000 - 53_389) < EPSILON);

    assert!(i32::abs(result.option_qd_pp.premier_total.droits_succession - 9_722) < EPSILON);
    assert!(i32::abs(result.option_qd_pp.cumul_etat - 72_777) < EPSILON);

    assert_eq!(result.check, CHECKED);
}

// En présence d'une donation-partage annulant les abattements c'est le contraire :
// les résultats sont bons quand il y a 100% d'usufruit mais diffèrent dès qu'il y a de la PP.
// En fait ils sont inférieurs au 1er décès mais supérieurs au 2ème décès et ils s'équilibrent
// dans les 2 cas avec uniquement de la PP (1/4 en PP et QD en PP)
#[test]
fn test_donations_partages() {
    let input = InputState {
        nb_enfants: 2,
        placements: 900_000,
        donations_partages: 400_000,
        age_vous: 65,
        age_conjoint: 64,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert!(i32::abs(result.option_totalite_us.premier_total.droits_succession - 50_389) < EPSILON);
    assert!(
        i32::abs(result.option_totalite_us.deuxieme_total.droits_succession - 86_389) < EPSILON
    );
    assert!(i32::abs(result.option_totalite_us.cumul_etat - 136_778) < EPSILON);

    // Différence inexpliquée de -10 000 au premier décès et de +10 000 au deuxième décès
    // => le total est exact
    assert!(
        i32::abs(result.option_1_4_pp.premier_total.droits_succession - 10_000 - 53_889) < EPSILON
    );
    assert!(
        i32::abs(result.option_1_4_pp.deuxieme_total.droits_succession + 10_000 - 118_889)
            < EPSILON
    );
    assert!(i32::abs(result.option_1_4_pp.cumul_etat - 172_778) < EPSILON);

    // Différence inexpliquée de -6 000 au premier décès et de +10 000 au deuxième décès
    // => le total présente une différence de +4 000
    assert!(
        i32::abs(result.option_1_4_pp_3_4_us.premier_total.droits_succession - 6_000 - 30_889)
            < EPSILON
    );
    assert!(
        i32::abs(result.option_1_4_pp_3_4_us.deuxieme_total.droits_succession + 10_000 - 118_889)
            < EPSILON
    );
    assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat + 4_000 - 149_778) < EPSILON);

    // Différence inexpliquée de -13 333 au premier décès et de +13 334 au deuxième décès
    // => le total est exact
    assert!(
        i32::abs(result.option_qd_pp.premier_total.droits_succession - 13_333 - 43_055) < EPSILON
    );
    assert!(
        i32::abs(result.option_qd_pp.deuxieme_total.droits_succession + 13_334 - 129_722) < EPSILON
    );
    assert!(i32::abs(result.option_qd_pp.cumul_etat - 172_777) < EPSILON);

    assert_eq!(result.check, CHECKED);
}
