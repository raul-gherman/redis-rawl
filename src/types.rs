use thiserror::Error;

/// Represents a redis RESP protocol response
/// https://redis.io/topics/protocol#resp-protocol-description
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    /// A nil response from the server.
    Nil,
    /// A status response that represents the string "OK".
    Okay,
    /// An integer response. Note that there are a few situations when redis actually returns a string for an integer.
    Int(i64),
    /// A simple string response.
    Status(String),
    /// A Bulk String reply.
    Bulk(Vec<u8>),
    /// An Array response of more data.
    Array(Vec<Value>),
}

pub type RedisResult<T> = std::result::Result<T, RedisError>;

#[derive(Error, Debug)]
#[error("RedisError (command: {command:?}, message: {message:?})")]
pub struct RedisError {
    pub command: String,
    pub message: String,
}

type Result<T> = std::result::Result<T, String>;

impl Value {
    pub fn try_into<T: ParseFrom<Self>>(self) -> Result<T> {
        T::parse_from(self)
    }
}

pub trait ParseFrom<T>: Sized {
    fn parse_from(value: T) -> Result<Self>;
}

impl ParseFrom<Value> for () {
    fn parse_from(value: Value) -> Result<Self> {
        match value {
            Value::Okay => Ok(()),
            v => Err(format!("Failed parsing {:?}", v)),
        }
    }
}

impl ParseFrom<Value> for i64 {
    fn parse_from(value: Value) -> Result<Self> {
        match value {
            Value::Int(n) => Ok(n),
            v => Err(format!("Failed parsing {:?}", v)),
        }
    }
}

impl ParseFrom<Value> for Vec<u8> {
    fn parse_from(value: Value) -> Result<Self> {
        match value {
            Value::Bulk(bytes) => Ok(bytes),
            v => Err(format!("Failed parsing {:?}", v)),
        }
    }
}

impl ParseFrom<Value> for String {
    fn parse_from(value: Value) -> Result<Self> {
        match value {
            Value::Okay => Ok("Ok".to_owned()),
            Value::Nil => Ok(String::new()),
            Value::Int(n) => Ok(format!("{}", n)),
            Value::Status(s) => Ok(s),
            Value::Bulk(bytes) => String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string()),
            v => Err(format!("Failed parsing {:?}", v)),
        }
    }
}

impl<T> ParseFrom<Value> for Vec<T>
where
    T: ParseFrom<Value>,
{
    fn parse_from(v: Value) -> Result<Self> {
        if let Value::Array(array) = v {
            let mut result = Vec::with_capacity(array.len());
            for e in array {
                result.push(T::parse_from(e)?);
            }
            return Ok(result);
        }
        Err(format!("Failed parsing {:?}", v))
    }
}
