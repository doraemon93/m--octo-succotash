use std::convert::TryFrom;
use std::io;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use actix::Message;
use tokio::codec::{Decoder, Encoder};

const HEADER_SIZE: usize = 4; // bytes

/// Type alias for BytesMut
pub type BytesMut = bytes::BytesMut;

// /// Message coming from the network
// #[derive(Debug, Message, Eq, PartialEq, Clone)]
// pub struct Request(pub BytesMut);

// /// Message going to the network
// #[derive(Debug, Message, Eq, PartialEq, Clone)]
// pub struct Response(pub BytesMut);

/// Codec for client -> server transport
///
/// Format:
/// ```ignore
/// Message size: u32
/// Message: [u8; Message size]
/// ```
///
/// The message format is described in the file [schemas/protocol.fbs][protocol]
///
/// [protocol]: https://github.com/witnet/witnet-rust/blob/master/schemas/protocol.fbs
#[derive(Debug, Message, Eq, PartialEq, Clone)]
pub struct P2PCodec;

/// Implement decoder trait for P2P codec
impl Decoder for P2PCodec {
    type Item = BytesMut;
    type Error = io::Error;

    /// Method to decode bytes to a request
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut ftb: Option<Self::Item> = None;
        let msg_len = src.len();
        if msg_len >= HEADER_SIZE {
            let mut header_vec = Cursor::new(&src[0..HEADER_SIZE]);
            let msg_size = header_vec.read_u32::<BigEndian>().unwrap() as usize;
            if msg_len >= msg_size + HEADER_SIZE {
                src.split_to(HEADER_SIZE);
                ftb = Some(src.split_to(msg_size));
            }
        }
        // If the message is incomplete, return without consuming anything.
        // This method will be called again when more bytes arrive.

        Ok(ftb)
    }
}

/// Implement encoder trait for P2P codec
impl Encoder for P2PCodec {
    type Item = BytesMut;
    type Error = io::Error;

    /// Method to encode a response into bytes
    fn encode(&mut self, bytes: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut encoded_msg = vec![];

        if bytes.len() > u32::max_value() as usize {
            log::error!("Maximum message size exceeded");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Message size {} bytes too big for u32", bytes.len()),
            ));
        }
        let header: u32 = u32::try_from(bytes.len()).unwrap();
        // push header with msg len
        encoded_msg.write_u32::<BigEndian>(header).unwrap();
        // push message
        encoded_msg.append(&mut bytes.to_vec());
        // push message to destination
        dst.unsplit(BytesMut::from(encoded_msg));
        Ok(())
    }
}
