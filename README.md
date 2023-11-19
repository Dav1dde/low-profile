Low Profile
===========

Low Profile is a no-std compatible HTTP server built for embedded,
originally started to power my esp32-s3.

Low Profile currently requires nightly 1.74+.

The framework is heavily inspired by Axum:

```rs
struct AppState;

impl FromRef<AppState> for &'static str {
    fn from_ref(_: &AppState) -> Self {
        "Hello State"
    }
}

#[embassy_executor::task]
async fn task(stack: &'static Stack<WifiDevice<'static>>) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    let router = Router::new()
        .post("/echo", |body: heapless::Vec<u8, 3>| async move { body })
        .get("/bar", |State(ret): State<&'static str>| async move { ret })
        .with_state(AppState);

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        if let Err(e) = socket.accept(80).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!("connection from: {:?}", socket.remote_endpoint());

        let (read, write) = socket.split();
        router.serve(read, write).await;

        socket.flush().await.unwrap();
        socket.close();
        Timer::after(Duration::from_millis(100)).await;
        socket.abort();
    }
}

```

**Note:** You probably shouldn't be using this .. yet
