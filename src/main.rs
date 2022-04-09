#[macro_use]
extern crate log;

use crate::config::{SERVER_ADDRESS, SERVER_PORT};
use crate::error::RsCraftError;
use crate::server::Server;
use tokio::spawn;

mod config;
mod error;
mod packet;
mod server;

#[tokio::main]
async fn main() -> Result<(), RsCraftError> {
    env_logger::init();

    let server = Server::new(SERVER_ADDRESS, SERVER_PORT).await?;

    info!("TCP listen on {}:{}", SERVER_ADDRESS, SERVER_PORT);

    loop {
        let mut connection = server.accept().await?;
        spawn(async move {
            // TODO
            connection.handle().await;
        });
    }
}
