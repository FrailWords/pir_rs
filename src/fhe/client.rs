mod pir_grpc;

use pir_grpc::pir_grpc_client::PirGrpcClient;
use pir_grpc::PirRequest;
use sunscreen::{
    types::{bfv::Signed},
    Ciphertext, Error, Params, PrivateKey,
    PublicKey, Runtime,
};
use bincode;

const SQRT_DATABASE_SIZE: usize = 10;

/// Client is a party that wants to look up a value in the database without
/// revealing what she looked up.
struct Client {
    pub public_key: PublicKey,
    private_key: PrivateKey,
    runtime: Runtime,
}

impl Client {
    pub fn setup(params: &Params) -> Result<Client, Error> {
        let runtime = Runtime::new(params)?;

        let (public_key, private_key) = runtime.generate_keys()?;

        Ok(Client {
            public_key,
            private_key,
            runtime,
        })
    }

    pub fn create_query(&self, index: usize) -> Result<(Ciphertext, Ciphertext), Error> {
        let col = index % SQRT_DATABASE_SIZE;
        let row = index / SQRT_DATABASE_SIZE;

        let mut col_query = [Signed::from(0); SQRT_DATABASE_SIZE];
        let mut row_query = [Signed::from(0); SQRT_DATABASE_SIZE];
        col_query[col] = Signed::from(1);
        row_query[row] = Signed::from(1);

        Ok((
            self.runtime.encrypt(col_query, &self.public_key)?,
            self.runtime.encrypt(row_query, &self.public_key)?,
        ))
    }

    pub fn check_response(&self, value: Ciphertext) -> Result<(), Error> {
        let value: Signed = self.runtime.decrypt(&value, &self.private_key)?;

        let value: i64 = value.into();

        println!("Client received {}", value);

        assert_eq!(value, 494);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client = PirGrpcClient::connect("http://[::1]:50051").await?;

    let params_request = tonic::Request::new({});
    let params = grpc_client.get_params(params_request).await?.into_inner().params;
    let params = Params::try_from_bytes(&params);

    let client = Client::setup(&params.unwrap()).unwrap();
    let (col_ciphertext, row_ciphertext) = client.create_query(94).unwrap();

    let request = tonic::Request::new(PirRequest {
        row_ciphertext: bincode::serialize(&row_ciphertext).unwrap(),
        col_ciphertext: bincode::serialize(&col_ciphertext).unwrap(),
        public_key: bincode::serialize(&client.public_key).unwrap(),
    });
    let pir_response = grpc_client.get_response(request).await?.into_inner();
    let response: Ciphertext = bincode::deserialize(&pir_response.response).unwrap();
    let result = client.check_response(response);

    Ok(())
}
