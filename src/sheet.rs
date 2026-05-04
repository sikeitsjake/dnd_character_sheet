use crate::{templates::*, user::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SheetField {
    BaseStrength = 0,
    Strength = 1,
    StrengthMod = 2,
    BAB = 3,
    Attack = 4,
}

impl SheetField {
    pub fn from_str(s: &str) -> Option<Self> {
        match (s) {
            "base_strength" => Some(Self::BaseStrength),
            "strength" => Some(Self::Strength),
            "strength_modifier" => Some(Self::StrengthMod),
            "bab" => Some(Self::BAB),
            "attack" => Some(Self::Attack),
            _ => None
        }
    }
}

pub const SHEET_NUM_STATS: usize = 5;

static SHEET_STR_MAP: [&'static str; SHEET_NUM_STATS] = [
    "base_strength",
    "strength",
    "strength_modifier",
    "bab",
    "attack",
];

static SHEET_DEP_MAP: [&'static [SheetField]; SHEET_NUM_STATS] = [
    &[SheetField::Strength],
    &[SheetField::StrengthMod],
    &[SheetField::Attack],
    &[SheetField::Attack],
    &[],
];

static SHEET_CALC_MAP: [fn(&[i32]) -> i32; SHEET_NUM_STATS] = [
    |vals| vals[SheetField::BaseStrength as usize],
    |vals| vals[SheetField::BaseStrength as usize],
    |vals| vals[SheetField::Strength as usize] / 2 - 5,
    |vals| vals[SheetField::BAB as usize],
    |vals| vals[SheetField::BAB as usize] + vals[SheetField::StrengthMod as usize],
];

pub fn calc_route(val_map: &mut [i32], field: SheetField, new_val: i32) -> Vec<StatField> {
    let mut updated_fields = Vec::new();

    val_map[field as usize] = new_val;
    let mut to_update_a = Vec::from(SHEET_DEP_MAP[field as usize]);
    let mut to_update_b = vec![];

    while !to_update_a.is_empty() {
        to_update_b.clear();

        for &field in &to_update_a {
            let new_val = SHEET_CALC_MAP[field as usize](&val_map);
            val_map[field as usize] = new_val;
            updated_fields.push(StatField {
                id: SHEET_STR_MAP[field as usize].to_string(),
                value: new_val,
            });
            to_update_b.extend_from_slice(SHEET_DEP_MAP[field as usize]);
        }

        std::mem::swap(&mut to_update_a, &mut to_update_b);
    }

    updated_fields
}
