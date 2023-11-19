use std::{net::Ipv4Addr, rc::Rc};

use embedded_io_adapters::tokio_1::FromTokio;
use low_profile::Service;
use tokio::task::LocalSet;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = low_profile::Router::new()
        .get("/", || async { "Hello World" })
        .post("/", |body: heapless::String<3>| async move { body });
    let router = Rc::new(router);

    let socket = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 8000)).await?;

    println!("Server listening on localhost:8000");
    let main = async move {
        loop {
            let (mut stream, addr) = socket.accept().await?;
            println!("Connection from: {addr}");

            let router = Rc::clone(&router);
            tokio::task::spawn_local(async move {
                let (reader, writer) = stream.split();
                let (reader, writer) = (FromTokio::new(reader), FromTokio::new(writer));

                if let Err(err) = router.serve(reader, writer).await {
                    println!("Could not serve request: {err:?}");
                };
            });
        }
    };

    LocalSet::new().run_until(main).await
}
