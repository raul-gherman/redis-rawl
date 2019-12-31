use resp::Decoder;
use std::fmt;
use std::io::{Error, ErrorKind};
use std::str;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

type RedisResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
    pub fn tcp_stream(&mut self) -> &mut TcpStream {
        self.reader.get_mut()
    }
    pub async fn get(&mut self, key: impl fmt::Display) -> io::Result<String> {
        let command = format!("get \"{}\"\r\n", key);
        self.tcp_stream().write_all(command.as_ref()).await?;
        // self.tcp_stream.read_u8().await?;

        let mut buffer = vec![0; 100];
        let ssss = self.tcp_stream().read(buffer.as_mut()).await?;
        unsafe {
            buffer.set_len(ssss);
        }
        let response = String::from_utf8(buffer).expect("stuff");
        Ok(response)
    }
    pub async fn decode(&mut self) -> io::Result<Value> {
        let mut res: Vec<u8> = Vec::new();
        self.reader.read_until(b'\n', &mut res).await?;

        let len = res.len();
        if len < 3 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("too short: {}", len),
            ));
        }
        if !is_crlf(res[len - 2], res[len - 1]) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid CRLF: {:?}", res),
            ));
        }

        let bytes = res[1..len - 2].as_ref();
        match res[0] {
            // Value::String
            b'+' => String::from_utf8(bytes.to_vec())
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))
                .map(Value::Status),
            // Value::Error
            b'-' => Err(match String::from_utf8(bytes.to_vec()) {
                Ok(value) => Error::new(ErrorKind::Other, value),
                Err(err) => Error::new(ErrorKind::InvalidData, err),
            }),
            // b'-' => parse_string(bytes).map(Value::Error),
            // Value::Integer
            b':' => parse_integer(bytes).map(Value::Int),
            // Value::Bulk
            b'$' => {
                let int = parse_integer(bytes)?;
                if int == -1 {
                    // Nil bulk
                    return Ok(Value::Nil);
                }
                if int < -1 || int >= RESP_MAX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("invalid bulk length: {}", int),
                    ));
                }

                let int = int as usize;
                let mut buf: Vec<u8> = vec![0; int + 2];
                self.reader.read_exact(buf.as_mut_slice()).await?;
                if !is_crlf(buf[int], buf[int + 1]) {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("invalid CRLF: {:?}", buf),
                    ));
                }
                buf.truncate(int);
                Ok(Value::Bulk(buf))
            }
            // Value::Array
            b'*' => {
                let int = parse_integer(bytes)?;
                if int == -1 {
                    // Null array
                    return Ok(Value::Nil);
                }
                if int < -1 || int >= RESP_MAX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("invalid array length: {}", int),
                    ));
                }

                let mut array: Vec<Value> = Vec::with_capacity(int as usize);
                for _ in 0..int {
                    let val = self.decode().await?;
                    array.push(val);
                }
                Ok(Value::Array(array))
            }
            prefix => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid RESP type: {:?}", prefix),
            )),
        }
    }
}

async fn hello_world() -> RedisResult<()> {
    let stream = TcpStream::connect("127.0.0.1:6379").await?;
    dbg!("connected");
    let mut con = RedisConnection::from(stream);
    let s = con.get("abc").await;
    dbg!(s);

    let mut tmp = Vec::with_capacity(10);
    tmp.push(43);
    let a = String::from_utf8(tmp).unwrap();
    let b = a.into_bytes();
    dbg!((&b, b.len(), b.capacity()));

    // stream.write_all(b"keys *\n").await?;
    // let mut buffer = [0; 100];
    // let ss = stream.read_u8().await?;
    // let r = stream.read(buffer.as_mut()).await?;
    // // Decoder::new(buffer[0..r].as_ref());
    // let s = str::from_utf8(&buffer[0..r])?;
    // dbg!(s);

    Ok(())
}

#[tokio::main]
async fn main() -> RedisResult<()> {
    hello_world().await
}

#[derive(PartialEq, Eq, Clone)]
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

#[inline]
fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
}

#[inline]
fn parse_string(bytes: &[u8]) -> io::Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|err| Error::new(ErrorKind::InvalidData, err))
}

#[inline]
fn parse_integer(bytes: &[u8]) -> io::Result<i64> {
    String::from_utf8(bytes.to_vec())
        .map_err(|err| Error::new(ErrorKind::InvalidData, err))
        .and_then(|value| {
            value
                .parse::<i64>()
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))
        })
}

/// up to 512 MB in length
const RESP_MAX_SIZE: i64 = 512 * 1024 * 1024;
const CRLF_BYTES: &'static [u8] = b"\r\n";
const NULL_BYTES: &'static [u8] = b"$-1\r\n";
const NULL_ARRAY_BYTES: &'static [u8] = b"*-1\r\n";
