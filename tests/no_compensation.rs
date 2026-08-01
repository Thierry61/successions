// Test d'intégration spécial pour vérifier les AV sans la gestion des récompenses
// (le but est de retrouver les mêmes résultats que le simulateur MACSF qui ne les implémente pas)
// Il peut être exécuté avec la commande : cargo test --test no_compensation --features no_compensation

#![cfg(feature = "no_compensation")]

use successions::data::{compute::compute, InputState, ResultState, EPSILON};

mod common;
use common::CHECKED;

#[test]
fn test_av_sans_recompense() {
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

    assert!(i32::abs(result.option_totalite_us.premier_total.droits_succession - 10_389) < EPSILON);
    assert!(i32::abs(result.option_totalite_us.cumul_etat - 125_778) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp.premier_total.droits_succession - 23_889) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp.cumul_etat - 161_778) < EPSILON);

    assert!(i32::abs(result.option_1_4_pp_3_4_us.premier_total.droits_succession - 125) < EPSILON);
    assert!(i32::abs(result.option_1_4_pp_3_4_us.cumul_etat - 138_014) < EPSILON);

    assert!(i32::abs(result.option_qd_pp.premier_total.droits_succession - 16_389) < EPSILON);
    assert!(i32::abs(result.option_qd_pp.cumul_etat - 161_778) < EPSILON);

    assert_eq!(result.check, CHECKED);
}
