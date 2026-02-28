use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    use serde::de::IgnoredAny;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Inner {
        BTreeMap(BTreeMap<String, IgnoredAny>),
        Vec(Vec<String>),
    }

    match Inner::deserialize(deserializer)? {
        Inner::BTreeMap(map) => {
            let keys = map.into_keys();
            let res: Result<Vec<T>, <T as FromStr>::Err> =
                keys.map(|key| <T as FromStr>::from_str(&key)).collect();
            res.map_err(serde::de::Error::custom)
        }
        Inner::Vec(set) => {
            let keys = set.into_iter();
            let res: Result<Vec<T>, <T as FromStr>::Err> =
                keys.map(|key| <T as FromStr>::from_str(&key)).collect();
            res.map_err(serde::de::Error::custom)
        }
    }
}
