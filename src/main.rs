#![allow(unused_imports)]
use core::str;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::Thread,
};

mod blocking;
mod server;

static DEFAULT_PORT: &str = "9092";

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    #[cfg(feature = "blocking")]
    sync_main();
    #[cfg(not(feature = "blocking"))]
    async_main();
}

fn sync_main() {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT)).unwrap();
    let threadpool = blocking::ThreadPool::new(4);
    for stream in listener.incoming() {
        match stream {
            Ok(conn) => {
                let _ = threadpool.execute(move || sync_run(conn));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn sync_run(mut conn: TcpStream) {
    println!("accepted new connection");
    let mut buf = [0; 1024];
    loop {
        if let Ok(n) = conn.read(&mut buf) {
            println!("received {} bytes", n);

            // exit
            if n == 0 {
                return;
            }

            #[cfg(feature = "diagnostic")]
            println!("\nReceived: {}", str::from_utf8(&buf[0..n]).unwrap());

            if let Ok(()) = conn.write_all(&buf[..n]) {
                #[cfg(feature = "diagnostic")]
                println!("sent {}", str::from_utf8(&buf[0..n]).unwrap());
            }
        }
        // reset buffer contents
        buf.fill(0);
    }
}

fn async_main() {
    // add tokio echo server here
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT))
            .await
            .unwrap();

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = [0; 1024];

                #[cfg(feature = "diagnostic")]
                println!("Handle connection!");

                loop {
                    #[cfg(feature = "diagnostic")]
                    print!("Waiting for new message:");

                    let n = socket
                        .read(&mut buf)
                        .await
                        .expect("failed to read data from socket");

                    if n == 0 {
                        return;
                    }

                    #[cfg(feature = "diagnostic")]
                    println!("\nReceived: {}", str::from_utf8(&buf[0..n]).unwrap());

                    socket
                        .write_all(&buf[0..n])
                        .await
                        .expect("failed to write data to socket");
                }
            });
        }
    });
}
