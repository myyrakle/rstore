# rstore

![](https://img.shields.io/badge/language-Rust-red) ![](https://img.shields.io/badge/version-0.1.1%20alpha-brightgreen) [![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/myyrakle/rstore/blob/master/LICENSE)

- simple Key-Value in-memory store
- HTTP or TCP server

## Just run (Local)

```bash
# TCP Server
cargo run --bin tcp

# HTTP Server
cargo run --bin http
```

## Start with Docker (HTTP)

run server

```bash
sudo docker run -p 13535:13535 myyrakle/rstore:http-0.1.1
```

ping

```bash
curl http://localhost:13535
```

set value

```bash
curl -X POST http://localhost:13535/value \
  -H "Content-Type: application/json" \
  -d '{"key": "example", "value": "42"}'
```

get value

```bash
curl -X GET http://localhost:13535/value?key=example
```

delete

```bash
curl -X DELETE http://localhost:13535/value?key=example
```

clear

```bash
curl -X DELETE http://localhost:13535/clear
```

## Start with Docker (TCP)

run server

```bash
sudo docker run -p 13535:13535 myyrakle/rstore:tcp-0.1.1
```

client code

```rust
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

    client.ping().await?;
    println!("PING PONG");

    client
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
```
