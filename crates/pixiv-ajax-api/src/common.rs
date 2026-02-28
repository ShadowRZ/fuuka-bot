//! Common types.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Restriction {
    General = 0,
    R18 = 1,
    R18G = 2,
}

impl<'de> Deserialize<'de> for Restriction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::General),
            1 => Ok(Self::R18),
            2 => Ok(Self::R18G),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value.into()),
                &"0, 1 or 2",
            )),
        }
    }
}

impl Serialize for Restriction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum AIType {
    Unknown = 0,
    NotAI = 1,
    AI = 2,
}

impl<'de> Deserialize<'de> for AIType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::NotAI),
            2 => Ok(Self::AI),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value.into()),
                &"0, 1 or 2",
            )),
        }
    }
}

impl Serialize for AIType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}
