use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    convert_to_timestamp, get_timespan, validate_range, ReservationError, ReservationQuery,
    ReservationStatus, Validator,
};

impl ReservationQuery {
    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref()).unwrap()
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        status: ReservationStatus,
        page: i32,
        is_desc: bool,
        page_size: i32,
    ) -> Self {
        Self {
            user_id: uid.into(),
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.unwrap())),
            end: Some(convert_to_timestamp(end.unwrap())),
            page,
            page_size,
            desc: is_desc,
            status: status as i32,
        }
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), ReservationError> {
        validate_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}
