use std::str;
use tokio::net::TcpStream;
use tokio::prelude::*;

async fn hello_world() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    stream.write_all(b"keys *\n").await?;
    let mut buffer = [0; 100];
    let r = stream.read(buffer.as_mut()).await?;
    let s = str::from_utf8(&buffer[0..r])?;

    dbg!(s);

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    hello_world().await
}
