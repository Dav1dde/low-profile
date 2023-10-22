use std::{net::Ipv4Addr, rc::Rc};

use embedded_io::adapters::FromTokio;
use low_profile::Service;
use tokio::task::LocalSet;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = low_profile::Router::new().get("/", || async { "Hello World" });
    let router = Rc::new(router);

    let socket = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 8000)).await?;

    let main = async move {
        loop {
            let (mut stream, addr) = socket.accept().await?;
            println!("Connection from: {addr}");

            let router = Rc::clone(&router);
            tokio::task::spawn_local(async move {
                let (reader, writer) = stream.split();

                router
                    .serve(FromTokio::new(reader), FromTokio::new(writer))
                    .await;
            });
        }
    };

    LocalSet::new().run_until(main).await
}
