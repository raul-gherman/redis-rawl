use std::io::{Error, ErrorKind};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::prelude::*;

use std::future::Future;
use std::pin::Pin;

use crate::Value;

/// reads the redis RESP responses into "Value"
/// ```
/// # use tokio::net::TcpStream;
/// # use tokio::io::BufReader;
/// # use self::resp::{Decoder, Value};
///
/// let mut stream = TcpStream::connect("127.0.0.1:6379").await?
/// stream.write_all("ping\r\n").await?;
/// let mut reader = BufReader::new(stream);
/// let value = decode(&mut reader).await?;
/// assert_eq!(value, Value::Status("PONG".to_string()));
/// ```
pub fn decode(
    reader: &mut BufReader<TcpStream>,
) -> Pin<Box<dyn '_ + Future<Output = io::Result<Value>>>> {
    Box::pin(async move {
        let mut res: Vec<u8> = Vec::new();
        reader.read_until(b'\n', &mut res).await?;

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
                reader.read_exact(buf.as_mut_slice()).await?;
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
                    let val = decode(reader).await?;
                    array.push(val);
                }
                Ok(Value::Array(array))
            }
            prefix => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid RESP type: {:?}", prefix),
            )),
        }
    })
}

#[inline]
fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
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
