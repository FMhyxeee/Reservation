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
    InvalidReservationId(i64),

    #[error("reservation conflict")]
    ConflictReservation(ReservationConflictInfo),

    #[error("db error: {0}")]
    DbError(sqlx::Error),

    #[error("reservation not found: {0}")]
    ReservationNotFound(i64),

    #[error("Not Found Row")]
    NotFoundRow,
}

impl PartialEq for ReservationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // TODO: this is a hack, need to find a better way to compare
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::ConflictReservation(v1), Self::ConflictReservation(v2)) => v1 == v2,
            (Self::InvalidResourceId(v1), Self::InvalidResourceId(v2)) => v1 == v2,
            (Self::InvalidReservationId(v1), Self::InvalidReservationId(v2)) => v1 == v2,
            (Self::InvalidTimespan, Self::InvalidTimespan) => true,
            (Self::InvalidUserId(v1), Self::InvalidUserId(v2)) => v1 == v2,
            (Self::ReservationNotFound(v1), Self::ReservationNotFound(v2)) => v1 == v2,
            (Self::Unknown, Self::Unknown) => true,
            (Self::NotFoundRow, Self::NotFoundRow) => true,
            _ => false,
        }
    }
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

impl From<ReservationError> for tonic::Status {
    fn from(e: ReservationError) -> Self {
        match e {
            ReservationError::Unknown => tonic::Status::unknown("unknown error"),
            ReservationError::InvalidTimespan => {
                tonic::Status::invalid_argument("invalid timespan")
            }
            ReservationError::InvalidUserId(v) => {
                tonic::Status::invalid_argument(format!("invalid userid: {}", v))
            }
            ReservationError::InvalidResourceId(v) => {
                tonic::Status::invalid_argument(format!("invalid resource id: {}", v))
            }
            ReservationError::InvalidReservationId(v) => {
                tonic::Status::invalid_argument(format!("invalid reservation id: {}", v))
            }
            ReservationError::ConflictReservation(v) => {
                tonic::Status::failed_precondition(format!("reservation conflict: {:?}", v))
            }
            ReservationError::DbError(e) => tonic::Status::internal(format!("db error: {}", e)),
            ReservationError::ReservationNotFound(v) => {
                tonic::Status::not_found(format!("reservation not found: {}", v))
            }
            ReservationError::NotFoundRow => tonic::Status::not_found("Not Found Row"),
        }
    }
}
