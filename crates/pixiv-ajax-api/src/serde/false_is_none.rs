use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either<T> {
        False(serde_bool::False),
        HasValue(T),
    }

    Either::deserialize(deserializer).map(|either| match either {
        Either::False(_) => None,
        Either::HasValue(value) => Some(value),
    })
}
