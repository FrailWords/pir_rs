mod pir_service;
mod pir_grpc;

use tonic::{transport::Server, Request, Response, Status, Streaming};

use pir_grpc::pir_grpc_server::{PirGrpc, PirGrpcServer};
use pir_grpc::{IndexTreeResponse, PirContext, PirRequest, PirResponse};

#[derive(Debug, Default)]
pub struct MyPirService {}

#[tonic::async_trait]
impl PirGrpc for MyPirService {

    async fn save_context(&self, request: Request<Streaming<PirContext>>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_response(&self, request: Request<PirRequest>) -> Result<Response<PirResponse>, Status> {
        todo!()
    }

    async fn get_index_tree(&self, request: Request<()>) -> Result<Response<IndexTreeResponse>, Status> {
        todo!()
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let pir_service = MyPirService::default();

    Server::builder()
        .add_service(PirGrpcServer::new(pir_service))
        .serve(addr)
        .await?;

    Ok(())
}