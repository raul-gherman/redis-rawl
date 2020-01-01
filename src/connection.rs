use crate::serialize;
use std::io::{Error, ErrorKind};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

impl RedisConnection {
    /// write a redis command into the socket.
    /// ```
    /// con.write("ping\r\n".as_ref())
    /// ```
    pub async fn write(&mut self, command: &[u8]) -> io::Result<()> {
        self.reader.get_mut().write_all(command).await
    }
    pub async fn read(&mut self) -> io::Result<Value> {
        serialize::decode(&mut self.reader).await
    }

    pub async fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        let cmd = format!("set {} {}\r\n", key, value);
        self.write(cmd.as_ref()).await?;
        match self.read().await? {
            Value::Okay => Ok(()),
            v => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v))),
        }
    }

    pub async fn get(&mut self, key: &str) -> io::Result<String> {
        let cmd = format!("get {}\r\n", key);
        self.write(cmd.as_ref()).await?;
        match self.read().await? {
            Value::Bulk(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|err| Error::new(ErrorKind::InvalidData, err)),
            v => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v))),
        }
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

/// Represents a redis RESP protcol response
/// https://redis.io/topics/protocol#resp-protocol-description
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    /// A nil response from the server.
    Nil,
    /// A status response which represents the string "OK".
    Okay,
    /// An integer response.  Note that there are a few situations
    /// in which redis actually returns a string for an integer.
    Int(i64),
    /// A simple string response.
    Status(String),
    /// A Bulk String reply.
    Bulk(Vec<u8>),
    /// An Array response of more data.
    Array(Vec<Value>),
}
