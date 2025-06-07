pub mod i64_as_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.parse::<i64>().map_err(serde::de::Error::custom)?)
    }
}

pub mod i64_as_string_option {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::i64_as_string;

    pub fn serialize<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => i64_as_string::serialize(value, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.parse::<i64>().is_ok() {
            Ok(Some(s.parse::<i64>().unwrap()))
        } else {
            Ok(None)
        }
    }
}

pub mod enum_from_str {
    use serde::{Deserialize, Deserializer};
    use std::{fmt::Display, str::FromStr};
    pub fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: Display,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(serde::de::Error::custom)
    }

    pub fn from_str_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => T::from_str(&s).map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}
