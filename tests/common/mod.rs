// Module commun aux tests d'intégration

use successions::data::CheckState;

// Vérification que le total de chaque option correspond aux actifs de départ
pub const CHECKED: CheckState = CheckState {
    option_totalite_us: true,
    option_1_4_pp: true,
    option_1_4_pp_3_4_us: true,
    option_qd_pp: true,
};
