use crate::message::MessageError;
use std::fmt;

#[derive(Debug)]
pub enum ServerError
{
    IO(tokio::io::Error),
    MessageError(MessageError)
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<tokio::io::Error> for ServerError
{
    fn from(e: tokio::io::Error) -> Self
    {
        ServerError::IO(e)
    }
}

impl From<MessageError> for ServerError
{
    fn from(e: MessageError) -> Self
    {
        ServerError::MessageError(e)
    }
}