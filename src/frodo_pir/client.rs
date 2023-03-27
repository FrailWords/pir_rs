mod frodo_pir_grpc;

use frodo_pir_grpc::frodo_pir_grpc_client::FrodoPirGrpcClient;
use frodo_pir_grpc::PirRequest;
use bincode;
use frodo_pir::api::{BaseParams, CommonParams, QueryParams, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut grpc_client = FrodoPirGrpcClient::connect("http://[::1]:50051").await?;

    let params_request = tonic::Request::new({});
    let params = grpc_client.get_params(params_request).await?.into_inner().params;
    let base_params: BaseParams = bincode::deserialize(&params).unwrap();
    println!("{}", base_params.get_dim());
    let common_params = CommonParams::from(&base_params);

    // Preprocess client queries before knowing query index (can be done offline)
    let mut query_params = QueryParams::new(&common_params, &base_params).unwrap();
    let query = query_params.prepare_query(2).unwrap();
    let query_serialized = bincode::serialize(&query).unwrap();

    let pir_request = tonic::Request::new(PirRequest {
        query: query_serialized,
    });
    let pir_response = grpc_client.get_response(pir_request).await?.into_inner().response;
    let resp: Response = bincode::deserialize(&pir_response).unwrap();
    let output = resp.parse_output_as_base64(&query_params);
    println!("{}", output);

    Ok(())
}