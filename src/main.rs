pub mod protocol;

extern crate serde;
use rstore::{
    client::{ClientResult, ConnectionConfig, RStoreClient},
    protocol::GetRequest,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Foo {
    pub user_id: String,
    pub user_name: String,
}

#[tokio::main]
async fn main() -> ClientResult<()> {
    let client = RStoreClient::new(ConnectionConfig {
        host: "0.0.0.0".to_string(),
        port: 13535,
        ..Default::default()
    });

    let response = client
        .get(GetRequest {
            key: "key".to_string(),
        })
        .await?;

    println!("Response: {:?}", response);

    Ok(())
}
