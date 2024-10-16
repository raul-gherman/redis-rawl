use tokio::io::{ AsyncBufReadExt, AsyncReadExt, BufReader };
use tokio::net::TcpStream;

use crate::types::Value;
use std::future::Future;
use std::pin::Pin;

/// reads the redis RESP responses from the socket into `Value`
/// ```
/// # use tokio::net::TcpStream;
/// # use tokio::io::BufReader;
/// # use self::resp::{Decoder, Value};
///
/// let mut stream = TcpStream::connect("127.0.0.1:6379").await?
/// stream.write_all("ping").await?;
/// let mut reader = BufReader::new(stream);
/// let value = decode(&mut reader).await?;
/// assert_eq!(value, Value::Status("PONG".to_string()));
/// ```

pub fn decode(
    reader: &mut BufReader<TcpStream>
) -> Pin<Box<dyn '_ + Future<Output = std::result::Result<Value, String>>>> {
    Box::pin(async move {
        let mut res: Vec<u8> = Vec::new();
        reader.read_until(b'\n', &mut res).await.map_err(|e| e.to_string())?;

        let len = res.len();
        if len < 3 {
            return Err(format!("too short: {}", len));
        }
        if !is_crlf(res[len - 2], res[len - 1]) {
            return Err(format!("invalid CRLF: {:?}", res));
        }

        let bytes = res[1..len - 2].as_ref();
        match res[0] {
            // Value::String
            b'+' =>
                match bytes {
                    OK_RESPONSE => Ok(Value::Okay),
                    bytes =>
                        String::from_utf8(bytes.to_vec())
                            .map_err(|e| e.to_string())
                            .map(Value::Status),
                }
            // Value::Error
            b'-' =>
                match String::from_utf8(bytes.to_vec()) {
                    Ok(value) => Err(value),
                    Err(e) => Err(e.to_string()),
                }
            // Value::Integer
            b':' => parse_integer(bytes).map(Value::Int),
            // Value::Bulk
            b'$' => {
                let int: i64 = parse_integer(bytes)?;
                if int == -1 {
                    // Nil bulk
                    return Ok(Value::Nil);
                }

                if !(-1..RESP_MAX_SIZE).contains(&int) {
                    return Err(format!("invalid bulk length: {}", int));
                }

                let int = int as usize;
                let mut buf: Vec<u8> = vec![0; int + 2];
                reader.read_exact(buf.as_mut_slice()).await.map_err(|e| e.to_string())?;
                if !is_crlf(buf[int], buf[int + 1]) {
                    return Err(format!("invalid CRLF: {:?}", buf));
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
                if !(-1..RESP_MAX_SIZE).contains(&int) {
                    return Err(format!("invalid array length: {}", int));
                }

                let mut array: Vec<Value> = Vec::with_capacity(int as usize);
                for _ in 0..int {
                    let val = decode(reader).await?;
                    array.push(val);
                }
                Ok(Value::Array(array))
            }
            prefix => Err(format!("invalid RESP type: {:?}", prefix)),
        }
    })
}

#[inline]
fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
}

#[inline]
fn parse_integer(bytes: &[u8]) -> std::result::Result<i64, String> {
    String::from_utf8(bytes.to_vec())
        .map_err(|e| e.to_string())
        .and_then(|value| value.parse::<i64>().map_err(|e| e.to_string()))
}

/// up to 512 MB in length
const RESP_MAX_SIZE: i64 = 512 * 1024 * 1024;
const OK_RESPONSE: &[u8] = &[79, 75];
