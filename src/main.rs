use std::{sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
}, time::Duration};

use anyhow::Error;
use tokio::{net::TcpListener, sync::Mutex};
use two_dis::{server::hanlde_connection, storage::Storage};
/*
    ! Note

    * \r\n (CRLF) would be the protocol's terminator

*/

#[tokio::main]
async fn main() -> Result<(), Error> {
    console_subscriber::init();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    
    println!("Two-dis server initializing ---------");
    println!("Redis server listening on {}", listener.local_addr()?);
    println!("----------------------------------");

    let main_storage = Arc::new(Mutex::new(
        Storage::default()
    ));

    let count = Arc::new(AtomicUsize::new(0));
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let count = Arc::clone(&count);
                let users = count.fetch_add(1, Ordering::Relaxed) + 1;

                println!("User connected | Concurrent users: {users}");
                let value = main_storage.clone();
                tokio::spawn(async move {
                    hanlde_connection(&mut stream, value).await;
                    
                    let users = count.fetch_sub(1, Ordering::Relaxed) - 1;
                    println!("User disconnected | Concurrent users: {users}");
                });
            }
            Err(error) => {
                eprintln!("Connection error: {error}");
            }
        }
    }
}
