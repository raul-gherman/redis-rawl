use crate::serialize;
use crate::types::{ParseFrom, RedisError, RedisResult, Value};
use tokio::io::{self, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

impl RedisConnection {
    pub async fn command<T: ParseFrom<Value>>(
        &mut self,
        command: String,
    ) -> RedisResult<T> {
        let command = format!("{command}\r\n");
        if let Err(io_err) = self.write(command.as_ref()).await {
            return Err(RedisError {
                message: io_err.to_string(),
                command,
            });
        }
        let value = match self.read().await {
            Err(message) => {
                return Err(RedisError { message, command });
            }
            Ok(v) => v,
        };
        value.try_into().map_err(|message| RedisError { message, command })
    }
    /// write a redis command into the socket.
    pub async fn write(
        &mut self,
        command: &[u8],
    ) -> io::Result<()> {
        self.reader.get_mut().write_all(command).await
    }
    /// read and parse a redis RESP protocol response.
    pub async fn read(&mut self) -> std::result::Result<Value, String> {
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
