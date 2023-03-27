use frodo_pir::api::*;

mod utils;
mod frodo_pir_grpc;

use tonic::{transport::Server, Request, Response, Status};
use frodo_pir_grpc::frodo_pir_grpc_server::{FrodoPirGrpc, FrodoPirGrpcServer};
use frodo_pir_grpc::{PirRequest, PirResponse, ServerParams};

pub struct FrodoPirService {
    db: Shard,
}

#[tonic::async_trait]
impl FrodoPirGrpc for FrodoPirService {
    async fn get_params(&self, _: Request<()>) -> Result<Response<ServerParams>, Status> {
        Ok(Response::new(ServerParams {
            params: bincode::serialize(&self.db.get_base_params()).unwrap(),
        }))
    }

    async fn get_response(&self, request: Request<PirRequest>) -> Result<Response<PirResponse>, Status> {
        let request = request.into_inner().query;
        let query: Query = bincode::deserialize(&request).unwrap();
        let server_response = self.db.respond(&query).unwrap();
        // let serialized_response = bincode::serialize(&result).unwrap();
        let pir_response = PirResponse {
            response: server_response
        };
        Ok(Response::new(pir_response))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Preprocessing performed by the server */

    // The LWE dimension to use
    let lwe_dim = 1572;
    // The number of rows in the database
    let m = 2u32.pow(2) as usize;
    // The length of each element in the database
    let ele_size = 2u32.pow(13) as usize;
    // The number of plaintext bits to use in each matrix element
    //   - 10 bits, for 16 ≤ log2(m) ≤ 18
    //   - 9 bits, for log2(m) ≤ 20
    // see Section 5 of paper for full details
    let plaintext_bits = 10usize;
    // Generates a random database
    let db_elements = utils::generate_db_elements(m, (ele_size + 7) / 8);
    let db = Shard::from_base64_strings(&db_elements, lwe_dim, m, ele_size, plaintext_bits).unwrap();

    let addr = "[::1]:50051".parse()?;
    let pir_service = FrodoPirService {
        db
    };

    Server::builder()
        .add_service(FrodoPirGrpcServer::new(pir_service))
        .serve(addr)
        .await?;

    Ok(())
}
