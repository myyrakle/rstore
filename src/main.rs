pub mod protocol;

extern crate serde;
use rstore::{
    client::{ClientResult, ConnectionConfig, RStoreClient},
    protocol::{GetRequest, SetRequest},
};

#[tokio::main]
async fn main() -> ClientResult<()> {
    let client = RStoreClient::new(ConnectionConfig {
        host: "0.0.0.0".to_string(),
        port: 13535,
        ..Default::default()
    });

    client.connect().await?;

    let _ = client.ping().await?;
    println!("PING PONG");

    let _ = client
        .set(SetRequest {
            key: "key".to_string(),
            value: "value".to_string(),
        })
        .await?;

    let response = client
        .get(GetRequest {
            key: "key".to_string(),
        })
        .await?;

    println!("Response: {:?}", response);

    Ok(())
}
