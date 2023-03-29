mod double_pir_grpc;

use std::io::Read;
use doublepir_rs::doublepir::{DoublePirClient, query};
use doublepir_rs::pir::PirClient;
use doublepir_rs::serializer::{DeserializeSlice, Serialize, State};
use double_pir_grpc::PirRequest;
use crate::double_pir_grpc::double_pir_grpc_client::DoublePirGrpcClient;

const SQRT_DATABASE_SIZE: usize = 10;

/// Client is a party that wants to look up a value in the database without
/// revealing what she looked up.
struct Client {
    pub pir_client: DoublePirClient,
}

struct ClientQuery {
    client_state: Vec<u8>,
    query_data: Vec<u8>,
    index: u64,
}

impl Client {
    pub fn setup(num_entries: u64, bits_per_entry: usize) -> Self {
        Client {
            pir_client: DoublePirClient::new(num_entries, bits_per_entry)
        }
    }

    pub fn create_query(&self, index: u64) -> ClientQuery {
        let (client_state, query_data) = self.pir_client.generate_query(index);
        ClientQuery {
            client_state,
            query_data,
            index,
        }
    }

    pub fn check_response(&self, query: &ClientQuery, response: &[u8]) -> Box<Vec<u8>> {
        let mut client_query_data = Vec::<State>::new();
        client_query_data.push(State::deserialize(&query.client_state));
        client_query_data.push(State::deserialize(&query.query_data));
        let response = self.pir_client.decode_response(response,
                                                       query.index,
                                                       &client_query_data.serialize());
        Box::new(response)
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client = DoublePirGrpcClient::connect("http://[::1]:50051").await?;

    let client = Client {
        pir_client : DoublePirClient::new(100, 8),
    };
    let query = client.create_query(5);
    let request = tonic::Request::new(PirRequest {
        query: query.query_data.clone()
    });
    let pir_response = grpc_client.get_response(request).await?.into_inner();
    let response = client.check_response(&query, &pir_response.response);
    println!("{:?}", response);

    Ok(())
}
