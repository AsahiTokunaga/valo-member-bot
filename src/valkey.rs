use redis::{
    AsyncCommands,
    Client
};
use redis::aio::MultiplexedConnection;
use anyhow::Result as AnyhowResult;

pub struct ValkeyModules;

impl ValkeyModules {
    pub async fn new(pass: String) -> AnyhowResult<MultiplexedConnection> {
        let client = Client::open(format!("redis://:{}@127.0.0.1/", pass))?;
        let mut connection = client.get_multiplexed_async_connection().await?;
        let pong: String = connection.ping().await?;
        println!("[ OK ] Redisに接続されました: {}", pong);
        Ok(connection)
    }
}   
