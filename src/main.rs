use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use anyhow::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use two_dis::protocal::RespFormatter;

/*
    ! Note

    * \r\n (CRLF) would be the protocol's terminator

*/

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let count = Arc::new(AtomicUsize::new(0));
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let count = Arc::clone(&count);
                let new_count =  count.fetch_add(1, Ordering::SeqCst);

                println!("accepted new connection {}", new_count);
                tokio::spawn(async move {
                    hanlde_connection(&mut stream).await;
                });
            }

            Err(_) => {
                continue;
            }
        }
    }
}

async fn hanlde_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 512];
    let pong = RespFormatter::simple_string("PONG");

    loop {
        match stream.read(&mut buffer).await {
            Ok(_) => {
                println!("Received bytes: {:?}", buffer);

                stream.write_all(pong.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
