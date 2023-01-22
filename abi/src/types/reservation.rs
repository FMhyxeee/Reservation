use std::ops::Range;

use chrono::{DateTime, FixedOffset, Utc};

use crate::{
    convert_to_timestamp, convert_to_utc_time, Reservation, ReservationError, ReservationStatus,
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
            id: "".to_string(),
            user_id: uid.into(),
            status: ReservationStatus::Pending as _,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ReservationError> {
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

    pub fn get_timestamp(&self) -> Range<DateTime<Utc>> {
        let start = convert_to_utc_time(self.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(self.end.as_ref().unwrap().clone());

        Range { start, end }
    }
}
