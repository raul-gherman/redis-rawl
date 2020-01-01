use crate::serialize;
use std::io::{Error, ErrorKind};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

macro_rules! redis_command {
    ($name:ident -> $returns:ty) => {
        pub async fn $name(&mut self, args: impl AsRef<str>)
         -> io::Result<$returns>
        {
            let cmd = format!(
                "{} {}\r\n",
                stringify!($name),
                args.as_ref()
            );
            self.command::<$returns>(cmd.as_ref()).await
        }
    };
}
impl RedisConnection {
    async fn command<T: ParseFrom<Value>>(&mut self, command: &[u8]) -> io::Result<T> {
        self.write(command).await?;
        let value = self.read().await?;
        value.try_into()
    }
    /// write a redis command into the socket.
    /// ```
    /// con.write("ping\r\n".as_ref())
    /// ```
    async fn write(&mut self, command: &[u8]) -> io::Result<()> {
        self.reader.get_mut().write_all(command).await
    }
    async fn read(&mut self) -> io::Result<Value> {
        serialize::decode(&mut self.reader).await
    }

    redis_command!(set    -> ());
    redis_command!(get    -> String);
    redis_command!(append -> i64);
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

impl Value {
    pub fn try_into<T: ParseFrom<Self>>(self) -> io::Result<T> {
        T::parse_from(self)
    }
}

pub trait ParseFrom<T>: Sized {
    fn parse_from(value: T) -> io::Result<Self>;
}

impl ParseFrom<Value> for () {
    fn parse_from(value: Value) -> io::Result<Self> {
        match value {
            Value::Okay => Ok(()),
            v => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v))),
        }
    }
}

impl ParseFrom<Value> for String {
    fn parse_from(value: Value) -> io::Result<Self> {
        match value {
            Value::Okay => Ok("Ok".to_owned()),
            Value::Nil => Ok(String::new()),
            Value::Int(n) => Ok(format!("{}", n)),
            Value::Status(s) => Ok(s),
            Value::Bulk(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|err| Error::new(ErrorKind::InvalidData, err)),
            v => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v))),
        }
    }
}

impl ParseFrom<Value> for i64 {
    fn parse_from(value: Value) -> io::Result<Self> {
        match value {
            Value::Int(n) => Ok(n),
            v => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v))),
        }
    }
}

impl<T> ParseFrom<Value> for Vec<T>
where
    T: ParseFrom<Value>,
{
    fn parse_from(v: Value) -> io::Result<Self> {
        if let Value::Array(array) = v {
            let mut result = Vec::with_capacity(array.len());
            for e in array {
                result.push(T::parse_from(e)?);
            }
            return Ok(result);
        }
        Err(Error::new(ErrorKind::InvalidData, format!("{:?}", v)))
    }
}
