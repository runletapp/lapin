# Lapin integration with tokio

This crate integrates lapin with tokio by using tokio's executor inside of lapin
for its internal operations and for consumer delegates.

```
use tokio_amqp::*;
use lapin::{Connection, ConnectionProperties, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    let channel = conn.create_channel().await?;

    // Rest of your program
}
```
