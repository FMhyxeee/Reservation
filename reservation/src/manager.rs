use abi::{ReservationError, ReservationId, Validator};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, PgPool, Row};

use crate::{ReservationManager, Rsvp};

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

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timestamp();

        // generate a insert sql for the reservation

        let id = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, status, note) VALUES ($1, $2, $3, $4::rsvp.reservation_status, $5) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(status.to_string())
            .bind(rsvp.note.clone())
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id;
        Ok(rsvp)
    }

    // if current status is pending, then change to confirmed
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        // let id =
        //     Uuid::parse_str(&id).map_err(|_| ReservationError::InvalidReservationId(id.clone()))?;
        id.validate()?;
        let status = sqlx::query_as(
            "UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(status)
    }

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        // let id =
        //     Uuid::parse_str(&id).map_err(|_| ReservationError::InvalidReservationId(id.clone()))?;
        id.validate()?;
        let status =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(status)
    }

    // 根据ID删除预约
    async fn delete(&self, id: ReservationId) -> Result<(), ReservationError> {
        // let id = Uuid::parse_str(&id).map_err(|_| ReservationError::InvalidReservationId(id))?;
        id.validate()?;
        let result = sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ReservationError::ReservationNotFound(id));
        } else {
            return Ok(());
        }
    }

    // 查看某个Reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        // let id = Uuid::parse_str(&id).map_err(|_| ReservationError::InvalidReservationId(id))?;
        id.validate()?;
        let status = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| ReservationError::ReservationNotFound(id))?;

        Ok(status)
    }

    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range = query.get_timespan();
        let status = abi::ReservationStatus::from_i32(query.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let rsvps = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;

        Ok(rsvps)
    }

    async fn filter(
        &self,
        filter: abi::ReservationFilter,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        let user_id = str_to_option(&filter.user_id);
        let resource_id = str_to_option(&filter.resource_id);
        let status = abi::ReservationStatus::from_i32(filter.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let rsvps = sqlx::query_as(
            "SELECT * FROM rsvp.filter($1, $2, $3::rsvp.reservation_status, $4, $5, $6::int)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(status.to_string())
        .bind(filter.cursor)
        .bind(filter.desc)
        .bind(filter.page_size as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rsvps)
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn str_to_option(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[cfg(test)]
mod tests {

    use abi::{
        Reservation, ReservationConflictInfo, ReservationFilterBuilder, ReservationQueryBuilder,
        ReservationStatus,
    };
    use chrono::FixedOffset;
    use prost_types::Timestamp;

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

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(rsvp.id != 0);
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
            println!("{err:?}");
            panic!("expect conflict reservation error");
        }
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_status_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert_eq!(rsvp.id, 1);

        let rsvp2 = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp2.status, ReservationStatus::Confirmed as i32)
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_status_not_pending_should_do_nothing() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert_eq!(rsvp.id, 1);

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, ReservationStatus::Confirmed as i32);

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, ReservationStatus::Confirmed as i32);

        println!("{rsvp:?}")
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_note_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert_eq!(rsvp.id, 1);

        let rsvp = manager
            .update_note(
                rsvp.id,
                "I'll arrive at 3pm. Please help to upgrade to executive room if possible"
                    .to_string(),
            )
            .await
            .unwrap();

        assert_eq!(
            rsvp.note,
            "I'll arrive at 3pm. Please help to upgrade to executive room if possible"
        )
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_delete_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert_eq!(rsvp.id, 1);

        assert!(manager.delete(rsvp.id).await.is_ok());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_delete_not_exists_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert_eq!(rsvp.id, 1);
        let id = rsvp.id;

        assert!(manager.delete(id).await.is_ok());
        let err = manager.delete(id).await.err().unwrap();
        assert!(manager.delete(id).await.is_err());
        assert_eq!(
            err.to_string(),
            ReservationError::ReservationNotFound(id).to_string()
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_get_reservation_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let id = rsvp.id;

        assert_eq!(id, 1);

        let rsvp2 = manager.get(id).await.unwrap();

        assert_eq!(rsvp, rsvp2);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_get_not_exist_id_should_err() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "first_id",
            "ocean-view-room-731",
            "2022-12-24T12:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let id = rsvp.id;

        assert_eq!(id, 1);
        let result: ReservationId = id;

        let err = manager.get(result + 1).await.err().unwrap();
        println!("{result:?}");
        assert_eq!(
            err.to_string(),
            ReservationError::ReservationNotFound(2).to_string()
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_query_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "hyx",
            "room-421",
            "2022-11-20T12:00:00-0700".parse().unwrap(),
            "2022-11-22T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let query = ReservationQueryBuilder::default()
            .user_id("hyx")
            // .resource_id("room-421")
            .start("2022-11-20T12:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2022-11-30T12:00:00-0700".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            // .page(1)
            // .desc(true)
            // .page_size(10)
            .build()
            .unwrap();

        println!("{query:?}");
        println!("{rsvp:?}");

        let result = manager.query(query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], rsvp);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_filter_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = Reservation::new_pending(
            "hyx",
            "room-421",
            "2022-11-20T12:00:00-0700".parse().unwrap(),
            "2022-11-22T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp1 = manager.reserve(rsvp).await.unwrap();

        let rsvp = Reservation::new_pending(
            "hyx",
            "room-421",
            "2022-11-22T12:00:00-0700".parse().unwrap(),
            "2022-11-24T12:00:00-0700".parse().unwrap(),
            "hello",
        );

        let rsvp2 = manager.reserve(rsvp).await.unwrap();

        let query = ReservationFilterBuilder::default()
            .user_id("hyx")
            .status(ReservationStatus::Pending as i32)
            .page_size(10)
            .build()
            .unwrap();

        let rsvps = manager.filter(query).await.unwrap();

        assert_eq!(rsvps.len(), 2);
        assert_eq!(rsvps, vec![rsvp1, rsvp2]);
    }
}
