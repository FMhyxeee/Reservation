mod config;
mod error;
#[allow(clippy::all, non_camel_case_types)]
mod pb;
mod types;
mod utils;

pub use config::*;
pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

pub type ReservationId = i64;
pub type UserId = String;
pub type ResourceId = String;

pub trait Validator {
    fn validate(&self) -> Result<(), ReservationError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), ReservationError> {
        if *self <= 0 {
            return Err(ReservationError::InvalidReservationId(*self));
        }
        Ok(())
    }
}
