
#![warn(rust_2018_idioms)]

use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use futures::FutureExt;
use std::env;
use std::error::Error;
use std::{thread, time};
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = Runtime::new().unwrap();
    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:443".to_string());
    let server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "10.10.10.10:443".to_string()); //git lfs server address


    let listen_addr2 = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:6666".to_string());
    let server_addr2 = env::args()
        .nth(2)
        .unwrap_or_else(|| "10.0.10.10:22".to_string()); //git ssh address

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    println!("Listening on: {}", listen_addr2);
    println!("Proxying to: {}", server_addr2);

    let listener = TcpListener::bind(listen_addr).await?;
    let listener2 = TcpListener::bind(listen_addr2).await?;

    let t1 = tokio::spawn(async move {
        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = transfer(inbound, server_addr.clone()).map(|r| {
                if let Err(e) = r {
                    println!("Failed to transfer; error={}", e);
                }
            });

            tokio::spawn(transfer);
        }
    });

    let t2 = tokio::spawn(async move {

        while let Ok((inbound, _)) = listener2.accept().await {
            let transfer = transfer(inbound, server_addr2.clone()).map(|r| {
                if let Err(e) = r {
                    println!("Failed to transfer; error={}", e);
                }
            });

            tokio::spawn(transfer);
        }
    });

    tokio::join!(t1, t2);

    Ok(())
}

async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
    let mut outbound = TcpStream::connect(proxy_addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
