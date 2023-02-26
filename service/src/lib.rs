use std::pin::Pin;

use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, Config,
    ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, QueryRequest, Reservation, ReserveRequest, ReserveResponse, UpdateRequest,
    UpdateResponse,
};

use futures::Stream;
use reservation::{ReservationManager, Rsvp};
use tonic::Status;

pub struct RsvpService {
    manager: ReservationManager,
}

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
    type queryStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

    type listenStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

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
