use abi::ReservationError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

use crate::{ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        rsvp.validate()?;

        // let start = convert_to_utc_time(rsvp.start.as_ref().unwrap().clone());
        // let end = convert_to_utc_time(rsvp.end.as_ref().unwrap().clone());

        // let Range{ start, end} = rsvp.get_timestamp();

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        // let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timestamp().into();

        // generate a insert sql for the reservation

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, status, note) VALUES ($1, $2, $3, $4::rsvp.reservation_status, $5) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(status.to_string())
            .bind(rsvp.note.clone())
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id.to_string();
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

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {

    use abi::{Reservation, ReservationConflictInfo};
    use chrono::FixedOffset;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-24T12:00:00-0700".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-28T12:00:00-0700".parse().unwrap();

        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            start,
            end,
            "I'll arrive at 3pm. Please help to upgrade to executive room if possible",
        );

        // let rsvp = abi::Reservation {
        //     id: "".to_string(),
        //     user_id: "first_id".to_string(),
        //     resource_id: "ocean-view-room-731".to_string(),
        //     start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
        //     end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
        //     status: abi::ReservationStatus::Pending as i32,
        //     note: "I'll arrive at 3pm. Please help to upgrade to executive room if possible"
        //         .to_string(),
        // };

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp1 = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let resp2 = Reservation::new_pending(
            "second_id",
            "ocean-view-room-731",
            "2022-12-25T12:00:00-0700".parse().unwrap(),
            "2022-12-27T12:00:00-0700".parse().unwrap(),
            "hello2",
        );

        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(resp2).await.unwrap_err();

        if let ReservationError::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.new.rid, "ocean-view-room-731");

            assert_eq!(info.new.start.to_rfc3339(), "2022-12-25T19:00:00+00:00");
            assert_eq!(info.new.end.to_rfc3339(), "2022-12-27T19:00:00+00:00");

            assert_eq!(info.old.rid, "ocean-view-room-731");
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-24T19:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
        } else {
            println!("{:?}", err);
            panic!("expect conflict reservation error");
        }
    }
}
