redis_rawl is a minimal Redis client library implementation.
It exposes a general purpose interface to Redis.

Forked from redis-raw `git@github.com:aminroosta/redis-raw-rs.git`, got up-to-date and adjusted.

```ini
[dependencies]
redis_rawl = "*"
```

# Basic Operation

`redis_rawl` exposes two API levels: a low- and a lower-level part!  
The `low-level` part does not expose all the functionality of redis and
might take some liberties in how it speaks the protocol.  The `lower-level`
part of the API allows you to express any request on the redis level.
You can fluently switch between both API levels at any point.

## Connection Handling

For connecting to redis you can use `tokio::net::TcpStream` which can be
converted to (or from) `RedisConnection`.

```rust
use redis_raw::RedisConnection;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // stablishes a TcpStream to redis
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    // RedisConnection can be converted to and from TcpStream
    let mut con: RedisConnection = stream.into();

    // we can use the same the lower level "command" fn
    con.command::<()>("set key value".to_owned()).await?;
    con.command::<i64>("append key !!!".to_owned()).await?;
    let value = con.command::<String>("get key".to_owned()).await?;

    assert_eq!(value, "value!!!");

    for i in 1..3 {
        con.command::<i64>(format!("zadd myset {} {}", i, i * i)).await?;
    }
    let myset = con.command::<Vec<String>>("zrange myset 0 -1".to_owned()).await?;

    assert_eq!(myset, vec!["1", "4"]);
    Ok(())
}

```

## Executing Lower-Level Commands

To execute lower-level commands you can use the `write()` and `read()` functions
which allow you to make redis requests and parse redis (RESP) responses.
These functions correspond to the underlying socket's read and write operations.

The `read()` function parses the RESP response as `redis_rawl::Value`.  
`Value` Represents a redis [RESP protcol response](https://redis.io/topics/protocol#resp-protocol-description).  

```rust
use redis_rawl::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<Value> {
   con.write("set key vvv").await?
   con.read().await
}
```

## Executing Low-Level Commands

The low-level interface is similar. The `command()` function does a
`write()` and a `read()` and converts the `Value` into requested type.

```rust
use redis_rawl::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<String> {
   con.command::<()>("set key value".to_owned()).await?;
   con.command::<i64>("append key !!!".to_owned()).await?;
   con.command::<String>("get key".to_owned()).await
}
```

Here is another example, to find out the correct result type see [redis docs](https://redis.io/commands).

```rust
use redis_rawl::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<Vec<String>> {
   for i in 1..10 {
       con.command::<i64>(format!("zadd myset {} {}", i, i*i)).await?;
   }
   con.command::<Vec<String>>("zrange myset 0 -1".to_owned()).await
}
```

The following return types are supported:
`()`, `i64`, `String`, `Vec<i64>`, and `Vec<String>`
