use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("unknown error")]
    Unknown,

    #[error("invalid timespan")]
    InvalidTimespan,

    #[error("db error: {0}")]
    DbError(#[from] sqlx::Error),
}
