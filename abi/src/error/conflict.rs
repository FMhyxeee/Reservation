// TODO: write a parser
// "Key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-25 19:00:00+00\",\"2022-12-27 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-24 19:00:00+00\",\"2022-12-28 19:00:00+00\"))."

use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    UnParsed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse() {
            Ok(Self::Parsed(conflict))
        } else {
            Ok(Self::UnParsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression to parse the string
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");

        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;

        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

pub struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression to parse the string
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();

        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            // 给数组中放入Option类型的值，在后面使用take()方法取出， 避免clone调用
            maps.push(Some(map));
        }

        if maps.len() != 2 {
            return Err(());
        }

        Ok(Self {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TEXT: &str = "Key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-25 19:00:00+00\",\"2022-12-27 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-24 19:00:00+00\",\"2022-12-28 19:00:00+00\")).";

    #[test]
    fn parse_datetime_should_work() {
        let dt = parse_datetime("2022-12-25 19:00:00+00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2022-12-25T19:00:00+00:00");
    }

    #[test]
    fn parsed_info_should_work() {
        let parsed = TEST_TEXT.parse::<ParsedInfo>().unwrap();
        assert_eq!(
            parsed.new.get("resource_id").unwrap(),
            "ocean-view-room-731"
        );
        assert_eq!(
            parsed.new.get("timespan").unwrap(),
            "\"2022-12-25 19:00:00+00\",\"2022-12-27 19:00:00+00\""
        );
        assert_eq!(
            parsed.old.get("resource_id").unwrap(),
            "ocean-view-room-731"
        );
        assert_eq!(
            parsed.old.get("timespan").unwrap(),
            "\"2022-12-24 19:00:00+00\",\"2022-12-28 19:00:00+00\""
        );
    }

    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-731".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2022-12-25 19:00:00+00\",\"2022-12-27 19:00:00+00\"".to_string(),
        );
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-731");
        assert_eq!(window.start.to_rfc3339(), "2022-12-25T19:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2022-12-27T19:00:00+00:00");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let conflict = TEST_TEXT.parse::<ReservationConflictInfo>().unwrap();
        match conflict {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-731");
                assert_eq!(conflict.new.start.to_rfc3339(), "2022-12-25T19:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-27T19:00:00+00:00");
                assert_eq!(conflict.old.rid, "ocean-view-room-731");
                assert_eq!(conflict.old.start.to_rfc3339(), "2022-12-24T19:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
            }
            ReservationConflictInfo::UnParsed(s) => {
                assert_eq!(s, "Key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-25 19:00:00+00\",\"2022-12-27 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-731, [\"2022-12-24 19:00:00+00\",\"2022-12-28 19:00:00+00\"])).");
            }
        }
    }
}
