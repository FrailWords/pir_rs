mod pir_grpc;

use pir_grpc::pir_grpc_client::PirGrpcClient;
use pir_grpc::PirRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PirGrpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(PirRequest {
        row: vec![],
        col: vec![],
    });

    let response = client.get_response(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
