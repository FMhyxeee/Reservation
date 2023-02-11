mod error;
#[allow(clippy::all, non_camel_case_types)]
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

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
