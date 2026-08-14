use serde::{Deserialize, Serialize, de::{Error, MapAccess, SeqAccess, Visitor}, ser::{SerializeSeq, SerializeStruct}};

use crate::{DiceExt, DiceExtSides};

/// Dice roll matrix mod for e.g. serde parsing, etc.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum DiceRollMatrixMod {
    Add(u8),
    /// `Div` ignores decimals entirely.
    Div(u8),
    /// `DivUp` "rounds" upward if dividing ends up with decimals.
    DivUp(u8),
    Mul(u8),
    Sub(u8),
}

impl <'de> Deserialize<'de> for DiceRollMatrixMod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        struct ModVis;
        impl<'de> Visitor<'de> for ModVis {
            type Value = DiceRollMatrixMod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a case-insensitive dice modifier thingy")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>,
            {
                if let Some((key, value)) = map.next_entry::<String, u8>()? {
                    match key.to_lowercase().as_str() {
                        "add" => Ok(DiceRollMatrixMod::Add(value)),
                        "div" => Ok(DiceRollMatrixMod::Div(value)),
                        "divup"|"div_up" => Ok(DiceRollMatrixMod::DivUp(value)),
                        "mul" => Ok(DiceRollMatrixMod::Mul(value)),
                        "sub" => Ok(DiceRollMatrixMod::Sub(value)),
                        _ => Err(A::Error::unknown_field(&key, &["add","div","divup","div_up","mul","sub"]))
                    }
                } else {
                    Err(A::Error::custom("empty modifier thingy"))
                }
            }
        }

        deserializer.deserialize_map(ModVis)
    }
}

trait DiceRollMatrixModifier {
    fn drmm(&self, drmm: DiceRollMatrixMod) -> i32;
}

impl DiceRollMatrixModifier for i32 {
    fn drmm(&self, drmm: DiceRollMatrixMod) -> i32 {
        match drmm {
            DiceRollMatrixMod::Add(v) => self + v as i32,
            DiceRollMatrixMod::Div(v) => self / v as i32,
            DiceRollMatrixMod::DivUp(v) => {
                let v = v as i32;
                (self + v - 1) / v
            }
            DiceRollMatrixMod::Mul(v) => self * v as i32,
            DiceRollMatrixMod::Sub(v) => self - v as i32,
        }
    }
}

/// Dice roll matrix for e.g. serde parsing, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiceRollMatrix {
    Exact { value: i32 },
    Percentage(u8), // de: "123%" -> 123_u8, ser: 123u8 -> "123%"
    Chance(u8, Box<DiceRollMatrix>),
    Multi(u8, u8),
    MultiWithMod(u8, u8, DiceRollMatrixMod),
}

impl DiceRollMatrix {
    pub fn roll(&self) -> i32 {
        match self {
            Self::Exact { value } => *value as i32,
            Self::Multi(num, sides) => (*num).d(*sides as DiceExtSides) as i32,
            Self::MultiWithMod(num, sides, drmm) =>
                Self::Multi(*num, *sides).roll().drmm(*drmm),
            Self::Percentage(p) => if 1.d100() <= *p { 1 } else { 0 },
            Self::Chance(p, drm) =>
                if 1.d100() > *p { 0 }
                else { drm.roll() },
        }
    }
}

impl From<i32> for DiceRollMatrix {
    fn from(value: i32) -> Self {
        Self::Exact { value }
    }
}

impl From<u8> for DiceRollMatrix {
    fn from(value: u8) -> Self {
        Self::Exact { value: value as i32 }
    }
}

impl From<&DiceRollMatrix> for bool {
    fn from(value: &DiceRollMatrix) -> Self {
        value.roll() > 0
    }
}

impl <'de> Deserialize<'de> for DiceRollMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct DRMVis;
        impl <'de> Visitor<'de> for DRMVis {
            type Value = DiceRollMatrix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("dice roll matrix thingy")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: Error,
            {
                if let Some(percstr) = v.strip_suffix('%') {
                    let n = percstr.parse::<u8>().map_err(E::custom)?;
                    return Ok(DiceRollMatrix::Percentage(n));
                }
                Err(E::custom(format!("'{v}' is an invalid string from DiceRollMatrix")))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where E: Error,
            {
                if v <= u8::MAX as u64 {
                    Ok(DiceRollMatrix::Exact { value: v as i32 })
                } else {
                    Err(E::custom(format!("'{v}' is too large for u8")))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>,
            {
                let (key, raw): (String, serde_json::Value) =
                    map.next_entry()?.ok_or_else(|| A::Error::custom("expceted a single-field object"))?;

                match key.as_str() {
                    "value" => {
                        let v: u8 = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        Ok(DiceRollMatrix::Exact { value: v as i32 })
                    }

                    "chance" => {
                        // expect: [pct, <DiceRollMatrix>]
                        let arr: Vec<serde_json::Value> = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        if arr.len() != 2 {
                            return Err(A::Error::custom("chance must be [pct, matrix]"));
                        }

                        let pct: u8 = serde_json::from_value(arr[0].clone()).map_err(A::Error::custom)?;
                        let inner: DiceRollMatrix = serde_json::from_value(arr[1].clone()).map_err(A::Error::custom)?;
                        Ok(DiceRollMatrix::Chance(pct, Box::new(inner)))
                    }

                    "range" => {
                        let arr: Vec<serde_json::Value> = serde_json::from_value(raw).map_err(A::Error::custom)?;
                        if arr.len() != 2 {
                            return Err(A::Error::custom("range must be [a, b]"));
                        }
                        let a: i32 = serde_json::from_value(arr[0].clone()).map_err(A::Error::custom)?;
                        let b: i32 = serde_json::from_value(arr[1].clone()).map_err(A::Error::custom)?;
                        let (modf, delta) = if a > b {
                            log::warn!("Range values require an U-turn from [{a},{b}] to [{b},{a}]. \
                            See to changing that although we'll let that pass… for now.");
                            let delta = a - b;
                            (a - 1 - delta, delta as u8)
                        } else {
                            let delta = b - a;
                            (b - 1 - delta, delta as u8)
                        };
                        if modf.abs() > u8::MAX as i32 {
                            return Err(A::Error::custom(format!("Modifier {modf} doesn't fit into u8…")));
                        }
                        Ok(if delta == 0 {
                            DiceRollMatrix::Exact { value: 0 }
                        } else {
                            DiceRollMatrix::MultiWithMod(1, delta, if modf < 0 { DiceRollMatrixMod::Sub(modf as u8)} else { DiceRollMatrixMod::Add(modf as u8) })
                        })
                    }

                    _ => Err(A::Error::custom(format!("unknown offender '{key}' in DiceRollMatrix")))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de>,
            {
                let a: u8 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing 1st element"))?;
                let b: u8 = seq.next_element()?.ok_or_else(|| A::Error::custom("missing 2nd element"))?;
                if let Some(modf) = seq.next_element::<DiceRollMatrixMod>()? {
                    Ok(DiceRollMatrix::MultiWithMod(a,b,modf))
                } else {
                    Ok(DiceRollMatrix::Multi(a,b))
                }
            }
        }

        deserializer.deserialize_any(DRMVis)
    }
}

impl Serialize for DiceRollMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        match self {
            Self::Exact { value } => {
                let mut st = serializer.serialize_struct("Exact", 1)?;
                st.serialize_field("value", value)?;
                st.end()
            }

            Self::Multi(a,b) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(a)?;
                seq.serialize_element(b)?;
                seq.end()
            }

            Self::MultiWithMod(a,b,m) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(a)?;
                seq.serialize_element(b)?;
                seq.serialize_element(m)?;
                seq.end()
            }

            Self::Percentage(p) => serializer.serialize_str(&format!("{p}%")),

            Self::Chance(p, drm) => {
                let mut st = serializer.serialize_struct("Chance", 1)?;
                st.serialize_field("chance", &(p, drm))?;
                st.end()
            }
        }
    }
}
