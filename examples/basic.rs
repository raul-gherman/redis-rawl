use redis_raw::RedisConnection;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // stablishes a TcpStream to redis
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    // RedisConnection can be converted to and from TcpStream
    let mut con: RedisConnection = stream.into();

    // we can use the same the lower level "command" fn
    con.command::<()>("set key value\r\n".to_owned()).await?;
    con.command::<i64>("append key !!!\r\n".to_owned()).await?;
    let value = con.command::<String>("get key\r\n".to_owned()).await?;

    assert_eq!(value, "value!!!");

    con.command::<i64>("zadd myset 1 one\r\n".to_owned())
        .await?;

    // or we can use redis named commands
    // these are thin wrappers around "command" fn
    // see full list of implemented commands in connection.rs
    con.set("key value").await?;
    con.append("key !!!").await?;
    let value = con.get("key").await?;

    assert_eq!(value, "value!!!");

    Ok(())
}
