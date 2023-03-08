use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, Config,
    ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use reservation::{ReservationManager, Rsvp};

use crate::{ReservationStream, RsvpService};

impl RsvpService {
    pub async fn from_config(config: &Config) -> Self {
        let pool = ReservationManager::from_config(&config.db).await;

        Self {
            manager: pool.unwrap(),
        }
    }
}

#[tonic::async_trait]
impl ReservationService for RsvpService {
    type queryStream = ReservationStream;

    type listenStream = ReservationStream;

    /// make a reservation
    async fn reserve(
        &self,
        request: tonic::Request<ReserveRequest>,
    ) -> Result<tonic::Response<ReserveResponse>, tonic::Status> {
        let req = request.into_inner();
        if req.reservation.is_none() {
            return Err(tonic::Status::invalid_argument("reservation is required"));
        }
        let rsvp = req.reservation.unwrap();

        let rsvp = self
            .manager
            .reserve(rsvp)
            .await
            .map_err(|e| tonic::Status::internal(format!("failed to reserve: {}", e)))?;

        Ok(tonic::Response::new(ReserveResponse {
            reservation: Some(rsvp),
        }))
    }
    /// confirm a pending reservation, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        _request: tonic::Request<ConfirmRequest>,
    ) -> Result<tonic::Response<ConfirmResponse>, tonic::Status> {
        todo!()
    }
    /// update the reservation note
    async fn update(
        &self,
        _request: tonic::Request<UpdateRequest>,
    ) -> Result<tonic::Response<UpdateResponse>, tonic::Status> {
        todo!()
    }
    /// cancel a reservation
    async fn cancel(
        &self,
        _request: tonic::Request<CancelRequest>,
    ) -> Result<tonic::Response<CancelResponse>, tonic::Status> {
        todo!()
    }
    /// get a reservation by id
    async fn get(
        &self,
        _request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        todo!()
    }

    /// query reservations by resource id, user id, status, start time, end time
    async fn query(
        &self,
        _request: tonic::Request<QueryRequest>,
    ) -> Result<tonic::Response<Self::queryStream>, tonic::Status> {
        todo!()
    }
    /// filter reservations, order by reservation id
    async fn filter(
        &self,
        _request: tonic::Request<FilterRequest>,
    ) -> Result<tonic::Response<FilterResponse>, tonic::Status> {
        todo!()
    }

    /// another system could monitor newly added/confirmed/cancelled reservations
    async fn listen(
        &self,
        _request: tonic::Request<ListenRequest>,
    ) -> Result<tonic::Response<Self::listenStream>, tonic::Status> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Deref, sync::Arc, thread};

    use abi::{
        reservation_service_server::ReservationService, Config, Reservation, ReserveRequest,
    };
    use sqlx::{types::Uuid, Connection, Executor};
    use tokio::runtime::Runtime;

    use crate::RsvpService;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref RT: Runtime = Runtime::new().unwrap();
    }

    struct TestConfig {
        config: Arc<Config>,
    }

    impl TestConfig {
        pub fn new() -> Self {
            let mut config = Config::load("../service/fixtures/config.yml").unwrap();

            let uuid = Uuid::new_v4();
            let dbname = format!("test-{}", uuid);
            config.db.dbname = dbname.clone();
            let serve_url = config.db.server_url();
            let url = config.db.to_url();

            thread::spawn(move || {
                // create database using config
                RT.block_on(async move {
                    let mut conn = sqlx::PgConnection::connect(&serve_url).await.unwrap();
                    let sql = format!(r#"CREATE DATABASE "{}""#, dbname);
                    conn.execute(sql.as_str()).await.unwrap();

                    let mut conn = sqlx::PgConnection::connect(&url).await.unwrap();
                    sqlx::migrate!("../migrations")
                        .run(&mut conn)
                        .await
                        .unwrap();
                    println!("run migrations ok");
                });
            })
            .join()
            .unwrap();

            Self {
                config: Arc::new(config),
            }
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            let server_url = self.config.db.server_url();
            let dbname = self.config.db.dbname.clone();

            thread::spawn(move || {
                RT.block_on(async move {
                    let mut conn = sqlx::PgConnection::connect(&server_url).await.unwrap();

                    // disconnect all active connections
                    sqlx::query(
                        format!("SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '{}' AND pid <> pg_backend_pid();", dbname).as_str())
                        .execute(&mut conn)
                        .await
                        .unwrap();
                    let result = conn
                        .execute(format!("DROP DATABASE {}", dbname).as_str())
                        .await
                        .unwrap();
                    println!("drop database result: {}", result.rows_affected());
                });
            });
        }
    }

    impl Deref for TestConfig {
        type Target = Config;

        fn deref(&self) -> &Self::Target {
            self.config.deref()
        }
    }

    #[tokio::test]
    async fn rpc_reserve_should_work() {
        let config = TestConfig::new();

        let service = RsvpService::from_config(&config).await;

        let rsvp = Reservation::new_pending(
            "hyx",
            "room-421",
            "2022-11-22T12:00:00-0700".parse().unwrap(),
            "2022-11-24T12:00:00-0700".parse().unwrap(),
            "hello",
        );
        let rsvp1 = rsvp.clone();

        let req = ReserveRequest {
            reservation: Some(rsvp),
        };

        let rsvp2 = service.reserve(tonic::Request::new(req)).await.unwrap();

        let rsvp2 = rsvp2.into_inner().reservation.unwrap();

        assert_eq!(rsvp1.id, rsvp2.id);
        assert_eq!(rsvp1.user_id, rsvp2.user_id);
        assert_eq!(rsvp1.resource_id, rsvp2.resource_id);
        assert_eq!(rsvp1.start, rsvp2.start);
        assert_eq!(rsvp1.end, rsvp2.end);
        assert_eq!(rsvp1.note, rsvp2.note);
    }
}
