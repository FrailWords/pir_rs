mod frodo_pir_grpc;

use frodo_pir_grpc::frodo_pir_grpc_client::FrodoPirGrpcClient;
use frodo_pir_grpc::PirRequest;
use bincode;
use frodo_pir::api::BaseParams;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client = FrodoPirGrpcClient::connect("http://[::1]:50051").await?;

    let params_request = tonic::Request::new({});
    let params = grpc_client.get_params(params_request).await?.into_inner().params;
    let params: BaseParams = bincode::deserialize(&params).unwrap();
    println!("{}", params.get_dim());

    Ok(())
}