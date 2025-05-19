use crate::serialize;
use crate::types::{
    ParseFrom,
    RedisError,
    RedisResult,
    Value,
};
use concat_strs_derive::concat_strs;
use tokio::io::{
    self,
    AsyncWriteExt,
    BufReader,
};
use tokio::net::TcpStream;

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

impl RedisConnection {
    pub async fn new() -> Self {
        let cache_endpoint = match std::env::var("REDIS_ENDPOINT") {
            Ok(cache_endpoint) => cache_endpoint,
            Err(_) => "127.0.0.1:6379".to_owned(),
        };
        let stream = match TcpStream::connect(&cache_endpoint).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("{e:#?}");
                std::process::exit(2)
            }
        };
        let redis_connection = match tokio::spawn(async move {
            RedisConnection {
                reader: BufReader::new(stream),
            }
        })
        .await
        {
            Ok(redis_connection) => redis_connection,
            Err(e) => {
                eprintln!("{e:#?}");
                std::process::exit(2);
            }
        };
        redis_connection
    }

    pub async fn command<T: ParseFrom<Value>>(
        &mut self,
        command: &str,
    ) -> RedisResult<T> {
        let command = concat_strs!(command, "\r\n");
        if let Err(io_err) = self
            .write(command.as_ref())
            .await
        {
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
        value
            .try_into()
            .map_err(|message| RedisError { message, command })
    }

    /// write a redis command into the socket.
    pub async fn write(
        &mut self,
        command: &[u8],
    ) -> io::Result<()> {
        self.reader
            .get_mut()
            .write_all(command)
            .await
    }

    /// read and parse a redis RESP protocol response.
    pub async fn read(&mut self) -> std::result::Result<Value, String> {
        serialize::decode(&mut self.reader).await
    }

    /// close redis client connection
    pub async fn close(self) -> io::Result<()> {
        self.reader
            .into_inner()
            .shutdown()
            .await
    }
}
