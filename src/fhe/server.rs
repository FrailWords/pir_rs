mod pir_service;
mod pir_grpc;

use tonic::{transport::Server, Request, Response, Status, Streaming};

use pir_grpc::pir_grpc_server::{PirGrpc, PirGrpcServer};
use pir_grpc::{IndexTreeResponse, PirRequest, PirResponse};


use sunscreen::{
    fhe_program,
    types::{bfv::Signed, Cipher},
    Ciphertext, CompiledFheProgram, Compiler, Error, FheProgramInput, Params, PrivateKey,
    PublicKey, Runtime,
};
use crate::pir_grpc::ServerParams;

const SQRT_DATABASE_SIZE: usize = 10;

#[fhe_program(scheme = "bfv")]
/// This program takes a user's query and looks up the entry in the database.
/// Queries are arrays containing a single 1 element at the
/// desired item's index and 0s elsewhere.
fn lookup(
    col_query: [Cipher<Signed>; SQRT_DATABASE_SIZE],
    row_query: [Cipher<Signed>; SQRT_DATABASE_SIZE],
    database: [[Signed; SQRT_DATABASE_SIZE]; SQRT_DATABASE_SIZE],
) -> Cipher<Signed> {
    // Safe Rust requires you initialize arrays with some value. Just put
    // put copies of col_query[0] and we'll overwrite them later.
    let mut col = [col_query[0]; SQRT_DATABASE_SIZE];

    // Perform matrix-vector multiplication with col_query to extract
    // Alice's desired column
    for i in 0..SQRT_DATABASE_SIZE {
        for j in 0..SQRT_DATABASE_SIZE {
            if j == 0 {
                col[i] = database[i][j] * col_query[j];
            } else {
                col[i] = col[i] + database[i][j] * col_query[j];
            }
        }
    }

    let mut sum = col[0] * row_query[0];

    // Dot product the result with the row query to get the result
    for i in 1..SQRT_DATABASE_SIZE {
        sum = sum + col[i] * row_query[i];
    }

    sum
}

/// This is the server that processes Alice's query.
struct SunServer {
    /// The compiled database query program
    pub compiled_lookup: CompiledFheProgram,

    /// The server's runtime
    runtime: Runtime,
}

impl SunServer {
    pub fn setup() -> Result<SunServer, Error> {
        let app = Compiler::new().fhe_program(lookup).compile()?;

        let runtime = Runtime::new(app.params())?;

        Ok(SunServer {
            compiled_lookup: app.get_program(lookup).unwrap().clone(),
            runtime,
        })
    }

    pub fn run_query(
        &self,
        col_query: Ciphertext,
        row_query: Ciphertext,
        public_key: &PublicKey,
    ) -> Result<Ciphertext, Error> {
        // Our database will consist of values between 400 and 500.
        let mut database = [[Signed::from(0); SQRT_DATABASE_SIZE]; SQRT_DATABASE_SIZE];
        let mut val = Signed::from(400);

        let mut state = [[0i64; SQRT_DATABASE_SIZE]; SQRT_DATABASE_SIZE];

        for i in 0..SQRT_DATABASE_SIZE {
            for j in 0..SQRT_DATABASE_SIZE {
                state[i][j] = val.into();
                database[i][j] = val;
                val = val + 1;
            }
        }
        println!("{:?}", state);

        let args: Vec<FheProgramInput> = vec![col_query.into(), row_query.into(), database.into()];

        let results = self.runtime.run(&self.compiled_lookup, args, public_key)?;

        Ok(results[0].clone())
    }
}

pub struct MyPirService {
    server: SunServer,
}


impl Default for MyPirService {
    fn default() -> Self {
        MyPirService {
            server: SunServer::setup().unwrap()
        }
    }
}


#[tonic::async_trait]
impl PirGrpc for MyPirService {
    async fn get_params(&self, _: Request<()>) -> Result<Response<ServerParams>, Status> {
        Ok(Response::new(ServerParams {
            params: self.server.compiled_lookup.metadata.params.to_bytes()
        }))
    }

    async fn get_response(&self, request: Request<PirRequest>) -> Result<Response<PirResponse>, Status> {
        let request = request.into_inner();
        let row_query: Ciphertext = bincode::deserialize(&request.row_ciphertext).unwrap();
        let col_query: Ciphertext = bincode::deserialize(&request.col_ciphertext).unwrap();
        let public_key: PublicKey = bincode::deserialize(&request.public_key).unwrap();
        let result = self.server.run_query(col_query, row_query, &public_key).unwrap();

        let serialized_response = bincode::serialize(&result).unwrap();
        let pir_response = PirResponse {
            response: serialized_response
        };
        Ok(Response::new(pir_response))
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
