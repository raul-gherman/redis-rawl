mod serialize;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

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
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    let mut con: RedisConnection = stream.into();
    con.write("ping\r\n".as_ref()).await?;
    let s = con.read().await?;
    dbg!(s);

    Ok(())
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    /// A nil response from the server.
    Nil,
    /// An integer response.  Note that there are a few situations
    /// in which redis actually returns a string for an integer which
    /// is why this library generally treats integers and strings
    /// the same for all numeric responses.
    Int(i64),
    /// An arbitary binary data.
    Bulk(Vec<u8>),
    /// An Array response of more data.
    Array(Vec<Value>),
    /// A status response.
    Status(String),
    /// A status response which represents the string "OK".
    Okay,
}
