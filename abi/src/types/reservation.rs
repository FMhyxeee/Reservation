use std::ops::Bound;

use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    FromRow, Row,
};

use crate::{
    convert_to_timestamp, convert_to_utc_time, get_timespan, Reservation, ReservationError,
    ReservationStatus, RsvpStatus, Validator,
};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn get_timestamp(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref()).unwrap()
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), ReservationError> {
        if self.user_id.is_empty() {
            return Err(ReservationError::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(ReservationError::InvalidResourceId(
                self.resource_id.clone(),
            ));
        }

        if self.start.is_none() || self.end.is_none() {
            return Err(ReservationError::InvalidTimespan);
        }

        let start = convert_to_utc_time(self.start.clone().unwrap());
        let end = convert_to_utc_time(self.end.clone().unwrap());

        if start > end {
            return Err(ReservationError::InvalidTimespan);
        }

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NavieRange<DateTime<Utc>> = range.into();
        assert!(range.start.is_some());
        assert!(range.end.is_some());
        let start = range.start.unwrap();
        let end = range.end.unwrap();

        let status: RsvpStatus = row.get("status");
        Ok(Self {
            id,
            user_id: row.try_get("user_id")?,
            status: ReservationStatus::from(status) as i32,
            resource_id: row.try_get("resource_id")?,
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            note: row.try_get("note")?,
        })
    }
}

struct NavieRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NavieRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        let start = f(range.start);
        let end = f(range.end);
        Self { start, end }
    }
}
