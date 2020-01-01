use redis_raw::{RedisConnection, RedisResult};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> RedisResult<()> {
    let stream = TcpStream::connect("127.0.0.1:6379")
        .await
        .expect("connected");
    let mut con: RedisConnection = stream.into();

    con.set("my_key \"my value\"").await.unwrap();
    con.append("my_key !!!").await.unwrap();

    let my_value = con.get("my_key").await.unwrap();

    assert_eq!(my_value, "my value!!!");
    dbg! {my_value};
    // let resp = con.write("set my_key my_value\r\n".as_ref()).await?;
    // assert_eq!(resp, Value::Okay);

    // dbg!(resp);
    // con.write("get my_key\r\n".as_ref()).await?;
    // let resp2 = con.read().await?;
    // dbg!(resp2);

    Ok(())
}
