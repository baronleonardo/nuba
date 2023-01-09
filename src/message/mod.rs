use std::{fmt, collections::HashMap, hash::Hash};

use once_cell::sync::OnceCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType
{
    NONE = 0,
    CMD,
    BUF
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MessageError
{
    pub msg: &'static str
}

pub struct Message
{
    buf: Vec<u8>,
    r#type: MessageType,
    body: (usize, usize), // start index, end index
    opts: (usize, usize),
}

impl Message
{
    pub fn new() -> Message
    {
        Message {
            buf: vec![],
            r#type: MessageType::NONE,
            body: (0, 0),
            opts: (0, 0),
        }
    }

    pub async fn from_buf(buf: Vec<u8>) -> Result<Message, MessageError>
    {
        static MSG_TYPE_MAX_LEN: usize = 3;
        static MIN_BUF_LEN: usize = 5;
        static ERROR_MSG: MessageError = MessageError { msg: "Invalid message buffer" };

        if buf.len() > MIN_BUF_LEN
        {
            let mut message = Message::new();
            message.buf = buf;

            match std::str::from_utf8(&message.buf[0..MSG_TYPE_MAX_LEN]) // + 1 for the space
            {
                Ok(str_msg_type) => if let Some(msg_type) = Self::get_message_type(str_msg_type)
                {
                    message.r#type = msg_type;

                    // make sure we have space
                    if message.buf[MSG_TYPE_MAX_LEN] != ' ' as u8 { return Err(ERROR_MSG) };

                    // check if there are options
                    match message.buf[MSG_TYPE_MAX_LEN + 1..].iter().position(|&ch| ch.is_ascii_whitespace())
                    {
                        Some(mut n) => {
                            n = MSG_TYPE_MAX_LEN + 1 + n;
                            message.body = (MSG_TYPE_MAX_LEN + 1, n);
                            message.opts = (n + 1, message.buf.len());
                        },
                        None => {
                            message.body = (MSG_TYPE_MAX_LEN + 1, message.buf.len());
                        }
                    };
                    return Ok(message);
                },

                Err(_) => return Err(ERROR_MSG)
            }
        }

        Err(ERROR_MSG)
    }

    pub fn r#type(&self) -> MessageType
    {
        self.r#type
    }

    pub fn body(&self) -> &str
    {
        std::str::from_utf8(&self.buf[self.body.0..self.body.1]).unwrap()
    }

    pub fn options(&self) -> &str
    {
        std::str::from_utf8(&self.buf[self.opts.0..self.opts.1]).unwrap()
    }

    fn get_message_type(str_msg_type: &str) -> Option<MessageType>
    {
        static INSTANCE: OnceCell<HashMap<&str, MessageType>> = OnceCell::new();
        let map = INSTANCE.get_or_init(|| {
            HashMap::<&str, MessageType>::from_iter([
                ("", MessageType::NONE),
                ("BUF", MessageType::BUF),
                ("CMD", MessageType::CMD),
            ])
        });

        map.get(str_msg_type).and_then(|&v| Some(v))
    }
}

//////////////////////////////////// tests ////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::message::{Message, MessageType};

    #[tokio::test]
    async fn valid_buf() {
        let buf: Vec<u8> = "CMD /read/file /usr/share/file1".as_bytes().to_vec();

        let message = Message::from_buf(buf).await;
        assert!(&message.is_ok());
        assert_eq!(&message.as_ref().unwrap().r#type(), &MessageType::CMD);
        assert_eq!(&message.as_ref().unwrap().body(), &"/read/file");
        assert_eq!(&message.as_ref().unwrap().options(), &"/usr/share/file1");
    }

    #[tokio::test]
    async fn valid_buf_with_opts() {
        let buf: Vec<u8> = "CMD /read/file".as_bytes().to_vec();

        let message = Message::from_buf(buf).await;
        assert!(&message.is_ok());
        assert_eq!(&message.as_ref().unwrap().r#type(), &MessageType::CMD);
        assert_eq!(&message.as_ref().unwrap().body(), &"/read/file");
        assert_eq!(&message.as_ref().unwrap().options(), &"");
    }

    #[tokio::test]
    async fn invalid_type() {
        let buf: Vec<u8> = "CM /read/file /usr/share/file1".as_bytes().to_vec();

        let message = Message::from_buf(buf).await;
        assert!(&message.is_err());
    }
    
    #[tokio::test]
    async fn invalid_type2() {
        let buf: Vec<u8> = "CMDD /read/file /usr/share/file1".as_bytes().to_vec();

        let message = Message::from_buf(buf).await;
        assert!(&message.is_err());
    }

    #[test]
    fn test_message_type()
    {
        println!("{}", Message::get_message_type("").unwrap());
        // println!("{}", Message::get_message_type("NONE").unwrap());
        println!("{}", Message::get_message_type("BUF").unwrap());
        println!("{}", Message::get_message_type("CMD").unwrap());
    }
}