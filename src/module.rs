use crate::{X_NUM_MODULES, Y_NUM_MODULES};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
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
    an,
    ku,
    we,
}
impl ShortModuleType {
    const fn make_long(self) -> ModuleType {
        match self {
            lk => ModuleType::Lkw,
            la => ModuleType::Lager,
            sä => ModuleType::Säge,
            fr => ModuleType::Fräse,
            st => ModuleType::Straße,
            ve => ModuleType::Versand,
            mo => ModuleType::MontageStation,
            be => ModuleType::Beschichtung,
            me => ModuleType::Messen,
            le => ModuleType::Leer,
            an => ModuleType::Anlieferung,
            ku => ModuleType::Kunde,
            we => ModuleType::Weg,
        }
    }
    const fn array_make_long<const N: usize>(short_array: [Self; N]) -> [ModuleType; N] {
        let mut long = [ModuleType::Lkw; N];

        let mut i = 0;
        while i < N {
            long[i] = short_array[i].make_long();
            i += 1;
        }
        long
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
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
    Anlieferung,
    Kunde,
    Weg,
}

impl ModuleType {}

use ShortModuleType::*;

pub const BOARD_MUDLES: [[ModuleType; X_NUM_MODULES]; Y_NUM_MODULES] = [
    ShortModuleType::array_make_long([lk, we, sä, la, fr, le, be, le, mo, le]),
    ShortModuleType::array_make_long([we, an, we, we, we, la, we, me, we, mo]),
    ShortModuleType::array_make_long([we, le, we, le, fr, le, we, we, we, mo]),
    ShortModuleType::array_make_long([we, le, we, le, we, le, we, we, ve, we]),
    ShortModuleType::array_make_long([we, an, we, le, la, le, we, we, le, we]),
    ShortModuleType::array_make_long([lk, we, we, we, we, we, we, we, we, ku]),
    ShortModuleType::array_make_long([le, le, fr, fr, le, me, le, ve, le, le]),
    ShortModuleType::array_make_long([le, le, le, le, le, le, le, le, le, le]),
    ShortModuleType::array_make_long([le, le, le, le, le, le, le, le, le, le]),
    ShortModuleType::array_make_long([le, le, le, le, le, le, le, le, le, le]),
];
