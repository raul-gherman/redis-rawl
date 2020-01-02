use redis_raw::{RedisConnection, Value}
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

    for i in 1..3 {
        con.command::<i64>(format!("zadd myset {} {}\r\n", i, i*i))
            .await?;
    }
    let myset = con.command::<Vec<String>>("zrange myset 0 -1\r\n".to_owned()).await?;

    assert_eq!(myset, vec!["1", "4"]);
    Ok(())
}
