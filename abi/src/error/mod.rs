mod conflict;

use thiserror::Error;

pub use conflict::*;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("unknown error")]
    Unknown,

    #[error("invalid timespan")]
    InvalidTimespan,

    #[error("invalid userid: {0}")]
    InvalidUserId(String),

    #[error("invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("invalid reservation id: {0}")]
    InvalidReservationId(String),

    #[error("reservation conflict")]
    ConflictReservation(ReservationConflictInfo),

    #[error("db error: {0}")]
    DbError(sqlx::Error),
}

impl From<sqlx::Error> for ReservationError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err = e.downcast_ref::<sqlx::postgres::PgDatabaseError>();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Self::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Self::DbError(sqlx::Error::Database(e)),
                }
            }
            _ => Self::DbError(e),
        }
    }
}
