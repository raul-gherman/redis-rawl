use redis_raw::{RedisConnection, Value};
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    let mut con: RedisConnection = stream.into();

    con.set("my_key", "my value").await?;
    let my_value = con.get("my_key").await?;
    dbg!(my_value);
    // let resp = con.write("set my_key my_value\r\n".as_ref()).await?;
    // assert_eq!(resp, Value::Okay);

    // dbg!(resp);
    // con.write("get my_key\r\n".as_ref()).await?;
    // let resp2 = con.read().await?;
    // dbg!(resp2);

    Ok(())
}
