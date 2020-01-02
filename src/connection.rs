use crate::serialize;
use crate::types::{ParseFrom, RedisError, RedisResult, Value};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

// macro_rules! redis_command {
//     ($name:ident -> $returns:ty) => {
//         pub async fn $name(
//             &mut self,
//             args: impl AsRef<str>
//         )
//          -> RedisResult<$returns>
//         {
//             let cmd = format!(
//                 "{} {}\r\n",
//                 stringify!($name),
//                 args.as_ref()
//             );
//             self.command::<$returns>(cmd).await
//         }
//     };
// }

impl RedisConnection {
    pub async fn command<T: ParseFrom<Value>>(&mut self, command: String) -> RedisResult<T> {
        if !command.ends_with("\r\n") {
            return Err(RedisError {
                message: "Commands must end with \\r\\n".to_owned(),
                command: command,
            });
        }
        if let Err(io_err) = self.write(command.as_ref()).await {
            return Err(RedisError {
                message: io_err.to_string(),
                command: command,
            });
        }
        let value = match self.read().await {
            Err(message) => {
                return Err(RedisError { message, command: command });
            }
            Ok(v) => v,
        };
        value
            .try_into()
            .map_err(|message| RedisError { message, command: command })
    }
    /// write a redis command into the socket.
    async fn write(&mut self, command: &[u8]) -> io::Result<()> {
        self.reader.get_mut().write_all(command).await
    }
    /// read and parse a redis RESP protocol response.
    async fn read(&mut self) -> std::result::Result<Value, String> {
        serialize::decode(&mut self.reader).await
    }
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
