// Tests de non regression, mais non validés par MACSF.
// Ils peuvent être exécutés avec la commande : cargo test --test regression

#![cfg(not(feature = "no_compensation"))]

mod common;
use common::CHECKED;
use successions::data::{compute::compute, InputState, ResultState};

// Les AV ont été validées par MACSF, mais sans la gestion des récompenses.
// Ce test implémente les récompenses.
#[test]
fn test_av_avec_recompense() {
    let input = InputState {
        nb_enfants: 2,
        placements: 900_000,
        age_vous: 65,
        age_conjoint: 64,
        av_vous_conjoint: 200_000,
        av_vous_enfants: 210_000,
        ordre_deces: true,
        av_conjoint_conjoint: 220_000,
        av_conjoint_enfants: 230_000,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert_eq!(result.premier_av_enfant.net, 105_000);
    assert_eq!(result.premier_av_survivant.net, 200_000);
    assert_eq!(result.deuxieme_av_enfant.net, 210_500);

    assert_eq!(
        result.option_totalite_us.premier_total.droits_succession,
        350
    );
    assert_eq!(result.option_totalite_us.cumul_etat, 91_738);

    assert_eq!(result.option_1_4_pp.premier_total.droits_succession, 8_138);
    assert_eq!(result.option_1_4_pp.cumul_etat, 128_026);

    assert_eq!(
        result.option_1_4_pp_3_4_us.premier_total.droits_succession,
        0
    );
    assert_eq!(result.option_1_4_pp_3_4_us.cumul_etat, 119_888);

    assert_eq!(result.option_qd_pp.premier_total.droits_succession, 2_480);
    assert_eq!(result.option_qd_pp.cumul_etat, 131_868);

    assert_eq!(result.check, CHECKED);
}

// Les PER ne sont pas gérés par MACSF
#[test]
fn test_per() {
    // Les 2 époux décèdent avant 70 ans
    let mut input = InputState {
        nb_enfants: 2,
        placements: 600_000,
        age_vous: 65,
        age_conjoint: 64,
        per_vous_conjoint: 300_000,
        per_conjoint_conjoint: 310_000,
        ordre_deces: true,
        deces_survivant_apres_70_ans: false,
        ignorer_couts_partage: true,
        ignorer_declaration_succession: true,
        ..Default::default()
    };
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert_eq!(result.premier_per, 300_000);
    assert_eq!(result.deuxieme_av_enfant.net, 154_500);

    assert_eq!(result.option_totalite_us.premier_total.droits_succession, 0);
    assert_eq!(result.option_totalite_us.cumul_etat, 77_388);

    assert_eq!(result.option_1_4_pp.premier_total.droits_succession, 1_730);
    assert_eq!(result.option_1_4_pp.cumul_etat, 94_118);

    assert_eq!(
        result.option_1_4_pp_3_4_us.premier_total.droits_succession,
        0
    );
    assert_eq!(result.option_1_4_pp_3_4_us.cumul_etat, 92_388);

    assert_eq!(result.option_qd_pp.premier_total.droits_succession, 0);
    assert_eq!(result.option_qd_pp.cumul_etat, 97_388);

    assert_eq!(result.check, CHECKED);

    // Décès de l'époux survivant après 70 ans
    input.deces_survivant_apres_70_ans = true;
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert_eq!(result.premier_per, 300_000);
    assert_eq!(result.deuxieme_per, 155_000);

    assert_eq!(result.option_totalite_us.premier_total.droits_succession, 0);
    assert_eq!(result.option_totalite_us.cumul_etat, 132_288);

    assert_eq!(result.option_1_4_pp.premier_total.droits_succession, 1_730);
    assert_eq!(result.option_1_4_pp.cumul_etat, 149_018);

    assert_eq!(
        result.option_1_4_pp_3_4_us.premier_total.droits_succession,
        0
    );
    assert_eq!(result.option_1_4_pp_3_4_us.cumul_etat, 147_288);

    assert_eq!(result.option_qd_pp.premier_total.droits_succession, 0);
    assert_eq!(result.option_qd_pp.cumul_etat, 152_288);

    assert_eq!(result.check, CHECKED);

    // Décès des 2 époux après 70 ans
    input.age_vous = 75;
    input.age_conjoint = 74;
    let mut result = ResultState::default();
    compute(&input, &mut result);

    assert_eq!(result.premier_per, 300_000);
    assert_eq!(result.deuxieme_per, 155_000);

    assert_eq!(
        result.option_totalite_us.premier_total.droits_succession,
        500
    );
    assert_eq!(result.option_totalite_us.cumul_etat, 132_788);

    assert_eq!(result.option_1_4_pp.premier_total.droits_succession, 1_730);
    assert_eq!(result.option_1_4_pp.cumul_etat, 149_018);

    assert_eq!(
        result.option_1_4_pp_3_4_us.premier_total.droits_succession,
        0
    );
    assert_eq!(result.option_1_4_pp_3_4_us.cumul_etat, 147_288);

    assert_eq!(result.option_qd_pp.premier_total.droits_succession, 0);
    assert_eq!(result.option_qd_pp.cumul_etat, 152_288);

    assert_eq!(result.check, CHECKED);
}
