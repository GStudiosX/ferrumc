use std::io::{Read, Write};
use tokio::io::AsyncWrite;
use crate::decode::{NetDecode, NetDecodeOpts, NetDecodeResult};
use crate::encode::{NetEncode, NetEncodeOpts, NetEncodeResult};
use crate::net_types::var_int::VarInt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LengthPrefixedVec<T>
{
    //pub length: VarInt,
    pub data: Vec<T>,
}

impl<T> LengthPrefixedVec<T>
{
    pub fn new(data: Vec<T>) -> Self {
        Self {
            //length: VarInt::new(data.len() as i32),
            data,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
}

impl<T> NetEncode for LengthPrefixedVec<T>
where
    T: NetEncode,
{
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> NetEncodeResult<()> {
        VarInt::new(self.data.len() as i32).encode(writer, opts)?;

        for item in &self.data {
            item.encode(writer, opts)?;
        }

        Ok(())
    }

    async fn encode_async<W: AsyncWrite + Unpin>(&self, writer: &mut W, opts: &NetEncodeOpts) -> NetEncodeResult<()> {
        VarInt::new(self.data.len() as i32).encode_async(writer, opts).await?;

        for item in &self.data {
            item.encode_async(writer, opts).await?;
        }

        Ok(())
    }
}
impl<T> NetDecode for LengthPrefixedVec<T>
where
    T: NetDecode,
{
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> NetDecodeResult<Self> {
        let length = VarInt::decode(reader, opts)?;

        let mut data = Vec::new();
        for _ in 0..length.val {
            data.push(T::decode(reader, opts)?);
        }

        Ok(Self {
            //length,
            data,
        })
    }
}
