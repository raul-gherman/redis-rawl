redis-raw-rs is a minimal Redis client library implementation.
It exposes a general purpose interface to Redis.

The crate is called `redis_raw` and you can depend on it via cargo:

```ini
[dependencies]
redis_raw = "*"
```

If you want to use the git version:

```ini
[dependencies]
redis_raw = { version = "*", git = "git@github.com:aminroosta/redis-raw-rs.git" }
```

# Basic Operation

`redis_raw` exposes two API levels: a low- and a lower-level part!  
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

async fn do_something()
 -> std::result::Result<(), Box<dyn std::error::Error>> {

   let stream = TcpStream::connect("127.0.0.1:6379").await?;
   let mut con: RedisConnection = stream.into();

    /* do something here */

    Ok(())
}
# fn main() {}
```

## Executing Lower-Level Commands

To execute lower-level commands you can use the `write()` and `read()` functions
which allow you to make redis requests and parse redis (RESP) responses.
These functions correspond to the underlying socket's read and write operations.

The `read()` function parses the RESP response as `redis_raw::Value`.  
`Value` Represents a redis [RESP protcol response](https://redis.io/topics/protocol#resp-protocol-description).  

```rust
use redis_raw::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<Value> {
   con.write("set key vvv\r\n").await?
   con.read().await
}
```

## Executing Low-Level Commands

The low-level interface is similar. The `command()` function does a
`write()` and a `read()` and converts the `Value` into requested type.

```rust
use redis_raw::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<String> {
   con.command::<()>("set key value\r\n".to_owned()).await?;
   con.command::<i64>("append key !!!\r\n".to_owned()).await?;
   con.command::<String>("get key\r\n".to_owned()).await
}
```

Here is another example, to find out the correct result type see [redis docs](https://redis.io/commands).

```rust
use redis_raw::{RedisConnection, RedisResult, Value }

fn do_something(con: &mut RedisConnection) -> RedisResult<Vec<String>> {
   for i in 1..10 {
       con.command::<i64>(format!("zadd myset {} {}\r\n", i, i*i)).await?;
   }
   con.command::<Vec<String>>("zrange myset 0 -1\r\n".to_owned()).await
}
```

The following return types are supported:
`()`, `i64`, `String`, `Vec<i64>`, and `Vec<String>`
