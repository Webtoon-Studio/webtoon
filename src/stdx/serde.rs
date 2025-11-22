use serde::{self, Deserializer};

pub fn u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl serde::de::Visitor<'_> for Visitor {
        type Value = u32;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("u32 or string containing a u32")
        }

        fn visit_u64<E>(self, v: u64) -> Result<u32, E>
        where
            E: serde::de::Error,
        {
            u32::try_from(v).map_err(E::custom)
        }

        fn visit_str<E>(self, v: &str) -> Result<u32, E>
        where
            E: serde::de::Error,
        {
            v.parse::<u32>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(Visitor)
}
