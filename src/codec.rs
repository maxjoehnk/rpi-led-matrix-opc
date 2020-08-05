use bytes::{Buf, BytesMut};
use futures_codec::{Decoder, Encoder};

const SET_PIXEL_COLORS: u8 = 0x00;

#[derive(Debug)]
pub enum Message {
    SetColors(u8, Vec<(u8, u8, u8)>),
}

pub struct OpcCodec;

impl Decoder for OpcCodec {
    type Item = Message;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let header = &src[..4];
        let channel = header[0];
        let command = header[1];
        let mut length_bytes = [0u8; 2];
        length_bytes.copy_from_slice(&header[2..4]);
        let length = u16::from_be_bytes(length_bytes) as usize + 4;
        if src.len() < length {
            return Ok(None);
        }
        let mut data = src.split_to(length).freeze();
        data.advance(4);

        if command == SET_PIXEL_COLORS {
            let mut colors = Vec::new();

            while !data.is_empty() {
                let red = data.get_u8();
                let green = data.get_u8();
                let blue = data.get_u8();

                colors.push((red, green, blue));
            }

            Ok(Some(Message::SetColors(channel, colors)))
        } else {
            Err(anyhow::anyhow!("Unknown command {}", command))
        }
    }
}

impl Encoder for OpcCodec {
    type Item = Message;
    type Error = anyhow::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
