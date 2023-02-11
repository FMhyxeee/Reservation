mod reservation;
mod reservation_query;
mod reservation_status;

use std::ops::Bound;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
pub use reservation::*;
pub use reservation_status::*;
use sqlx::postgres::types::PgRange;

use crate::{convert_to_utc_time, ReservationError};

pub fn validate_range(
    start: Option<&Timestamp>,
    end: Option<&Timestamp>,
) -> Result<(), ReservationError> {
    if start.is_none() || end.is_none() {
        return Err(ReservationError::InvalidTimespan);
    }

    let start = start.unwrap();
    let end = end.unwrap();

    if start.seconds > end.seconds {
        return Err(ReservationError::InvalidTimespan);
    }

    Ok(())
}

pub fn get_timespan(
    start: Option<&Timestamp>,
    end: Option<&Timestamp>,
) -> Result<PgRange<DateTime<Utc>>, ReservationError> {
    validate_range(start, end)?;

    let start = convert_to_utc_time(start.unwrap().clone());
    let end = convert_to_utc_time(end.unwrap().clone());

    Ok(PgRange {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
    })
}
