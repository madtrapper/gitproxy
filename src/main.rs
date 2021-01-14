
#![warn(rust_2018_idioms)]

use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use futures::FutureExt;
use std::env;
use std::error::Error;
use std::{thread, time};

async fn t1() -> Result<u32, u32> {
    let ten_secs = time::Duration::from_secs(3);
    thread::sleep(ten_secs);
    println!("t1 done");
    Ok(10)
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let t1_func = t1();

    let r = rt.spawn(t1_func);
   

    let r1 = r.await?;

    println!("{:?}, --done--", r1);

    Ok(())
}
*/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:443".to_string());
    let server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "172.16.12.100:443".to_string());
        //.unwrap_or_else(|| "172.16.12.11:443".to_string());
        //.unwrap_or_else(|| "172.16.12.11:22".to_string());

    let listen_addr2 = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:6666".to_string());
    let server_addr2 = env::args()
        .nth(2)
        .unwrap_or_else(|| "172.16.12.100:22".to_string());

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

    t1.await?;
    t2.await?;

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
