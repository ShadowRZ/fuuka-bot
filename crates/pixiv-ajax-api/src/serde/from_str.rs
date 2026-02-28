use serde::{Deserialize, Deserializer, Serializer};
use std::{fmt::Display, str::FromStr};

pub(crate) fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: FromStr + Display,
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + Display,
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    T::from_str(&string).map_err(|_| {
        serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&string),
            &std::any::type_name::<T>(),
        )
    })
}

pub mod option {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::{fmt::Display, str::FromStr};

    pub(crate) fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: FromStr + Display,
        S: Serializer,
    {
        match value.as_ref() {
            Some(value) => serializer.serialize_some(&value.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr + Display,
        D: Deserializer<'de>,
    {
        let string = Option::<String>::deserialize(deserializer)?;

        string
            .map(|string| {
                T::from_str(&string).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(&string),
                        &std::any::type_name::<T>(),
                    )
                })
            })
            .transpose()
    }
}
