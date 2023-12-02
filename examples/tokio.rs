use std::{net::Ipv4Addr, rc::Rc};

use embedded_io_adapters::tokio_1::FromTokio;
use low_profile::{alloc, extract::Path, heapless::Json, Segment, Service};
use tokio::task::LocalSet;

#[derive(serde::Deserialize)]
struct Body {
    content: heapless::String<128>,
}

#[derive(serde::Serialize)]
struct Response {
    response: heapless::String<128>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = low_profile::Router::new()
        .get("/", || async { "hello world" })
        .post("/", |body: heapless::String<3>| async move { body })
        .get(
            ("param", heapless::String::<3>::segment()),
            |Path((_, p))| async move { p },
        )
        // JSON using `serde-json-core` allocation free.
        .post("/json", |Json(body): Json<Body, 256>| async move {
            Json::<_, 256>(Response {
                response: body.content,
            })
        })
        // JSON using `serde_json` with `alloc`.
        .post(
            "/json/alloc",
            |alloc::Json(body): alloc::Json<Body>| async move {
                alloc::Json(Response {
                    response: body.content,
                })
            },
        );
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
