use crate::serialize;
use crate::types::{ParseFrom, RedisError, RedisResult, Value};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

macro_rules! redis_command {
    ($name:ident -> $returns:ty) => {
        pub async fn $name(
            &mut self,
            args: impl AsRef<str>
        )
         -> RedisResult<$returns>
        {
            let cmd = format!(
                "{} {}\r\n",
                stringify!($name),
                args.as_ref()
            );
            self.command::<$returns>(cmd).await
        }
    };
}

impl RedisConnection {
    /// write a redis command into the socket.
    /// ```
    /// con.write("ping\r\n".as_ref())
    /// ```
    async fn write(&mut self, command: &[u8]) -> io::Result<()> {
        self.reader.get_mut().write_all(command).await
    }
    async fn read(&mut self) -> std::result::Result<Value, String> {
        serialize::decode(&mut self.reader).await
    }

    pub async fn command<T: ParseFrom<Value>>(&mut self, command: String) -> RedisResult<T> {
        if let Err(io_err) = self.write(command.as_ref()).await {
            return Err(RedisError {
                message: io_err.to_string(),
                command: command,
            });
        }
        let value = match self.read().await {
            Err(message) => {
                return Err(RedisError { message, command });
            }
            Ok(v) => v,
        };
        value
            .try_into()
            .map_err(|message| RedisError { message, command })
    }

    redis_command!(set    -> ());
    redis_command!(get    -> String);

    redis_command!(append       -> i64);
    redis_command!(auth         -> String);
    redis_command!(bgrewriteaof -> String);
    redis_command!(bgsave       -> String);
    redis_command!(BITCOUNT     -> i64);
    redis_command!(BITFIELD     -> i64);
}

pub struct RedisConnection {
    reader: BufReader<TcpStream>,
}

impl From<TcpStream> for RedisConnection {
    fn from(tcp_stream: TcpStream) -> Self {
        RedisConnection {
            reader: BufReader::new(tcp_stream),
        }
    }
}

impl Into<TcpStream> for RedisConnection {
    fn into(self) -> TcpStream {
        self.reader.into_inner()
    }
}
