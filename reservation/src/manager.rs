use abi::convert_to_utc_time;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, Row};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTimespan);
        }

        let start = convert_to_utc_time(rsvp.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(rsvp.end.as_ref().unwrap().clone());

        // let status = abi::ReservationStatus::from_i32(rsvp.status).unwrap_or(abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        // generate a insert sql for the reservation

        let id: i64 = sqlx::query(
            "INSERT INTO reservation (user_id, resource_id, timespan, status, note) VALUES ($1, $2, $3, $4, $5) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.status)
            .bind(rsvp.note.clone())
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn change_status(
        &self,
        _id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}
