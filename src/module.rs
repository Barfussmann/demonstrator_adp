use crate::{X_NUM_MODULES, Y_NUM_MODULES};

#[allow(non_camel_case_types)]
enum ShortModuleType {
    lk,
    la,
    sä,
    fr,
    st,
    ve,
    mo,
    be,
    me,
    le,
}
enum LongModuleType {
    Lkw,
    Lager,
    Säge,
    Fräse,
    Straße,
    Versand,
    MontageStation,
    Beschichtung,
    Messen,
    Leer,
}

use ShortModuleType::*;

// const BOARD_MUDLES: [[ShortModuleType; X_NUM_MODULES]; Y_NUM_MODULES] = [
//     [lk, le, ],
//     [],
//     [],
//     [],
//     [],
//     [],
//     [],
//     [],
//     [],
//     [],
// ]
