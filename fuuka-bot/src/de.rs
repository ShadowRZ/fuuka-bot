use serde::{Deserialize, Deserializer};
use time::macros::offset;

pub(crate) fn deserialize_unix_timestamp<'de, D>(
    deserializer: D,
) -> Result<time::OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    use time::OffsetDateTime;

    let ts = i64::deserialize(deserializer)?;
    let dt = OffsetDateTime::from_unix_timestamp(ts)
        .map(|ts| ts.to_offset(offset!(+8)))
        .map_err(serde::de::Error::custom)?;
    Ok(dt)
}
