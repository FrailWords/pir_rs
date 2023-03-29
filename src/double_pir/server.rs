use doublepir_rs::doublepir::DoublePirServer;
use doublepir_rs::pir::PirServer;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::double_pir_grpc::double_pir_grpc_server::{DoublePirGrpc, DoublePirGrpcServer};
use crate::double_pir_grpc::{PirRequest, PirResponse, ServerParams};

mod double_pir_grpc;

struct DoublePirService {
    server : DoublePirServer
}

impl DoublePirService {
    pub fn setup() -> Self {
        let server = DoublePirServer::new(100, 8);
        DoublePirService {
            server
        }
    }
}

#[tonic::async_trait]
impl DoublePirGrpc for DoublePirService {
    async fn get_params(&self, request: Request<()>) -> Result<Response<ServerParams>, Status> {
        Ok(Response::new(ServerParams{
            params: String::from("").into_bytes()
        }))
    }

    async fn get_response(&self, request: Request<PirRequest>) -> Result<Response<PirResponse>, Status> {
        let query = request.into_inner().query;
        let response = self.server.answer(&query);
        Ok(Response::new(PirResponse {
            response
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let pir_service = DoublePirService::setup();
    Server::builder()
        .add_service(DoublePirGrpcServer::new(pir_service))
        .serve(addr)
        .await?;

    Ok(())
}
