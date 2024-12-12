use bitflags::bitflags;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use zstd::{bulk::Compressor, decode_all};

bitflags! {
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const Alpha = 0b00000001;
    }
}

#[derive(Debug)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u8,
}

impl Header {
    pub const MAGIC: [u8; 4] = *b"BOOP";
    pub const VERSION: u8 = 1;

    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }

    pub fn is_supported(&self) -> bool {
        self.version <= Self::VERSION
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(Self::MAGIC.len() + 1);

        bytes.put(&Self::MAGIC[..]);
        bytes.put_u8(Self::VERSION);

        bytes.freeze()
    }

    pub fn from_bytes(mut src: Bytes) -> Result<Self, Error> {
        if src.remaining() < size_of::<Self>() {
            return Err(Error::MalformedHeader);
        }

        let mut magic = [0u8; 4];
        src.copy_to_slice(&mut magic);
        let version = src.get_u8();

        Ok(Self { magic, version })
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
        }
    }
}

#[derive(Debug)]

pub struct InformationV1 {
    pub flags: Flags,
    pub width: u32,
    pub height: u32,
}

impl InformationV1 {
    pub fn new(flags: Flags, width: u32, height: u32) -> Self {
        Self {
            flags,
            width,
            height,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(size_of::<u32>() * 3);

        bytes.put_u32_le(self.flags.bits());
        bytes.put_u32_le(self.width);
        bytes.put_u32_le(self.height);

        bytes.freeze()
    }

    pub fn from_bytes(mut src: Bytes) -> Result<Self, Error> {
        if src.remaining() < size_of::<Self>() {
            return Err(Error::MalformedBody);
        }

        let flags = Flags::from_bits_retain(src.get_u32_le());
        let width = src.get_u32_le();
        let height = src.get_u32_le();

        Ok(Self {
            flags,
            width,
            height,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    MalformedHeader,
    #[error("")]
    Unsupported,
    #[error("")]
    MalformedBody,
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct BoopImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl BoopImage {
    // Creates a new BoopImage from raw RGB8 data
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn as_raw(&self) -> &[u8] {
        &self.data
    }

    pub fn into_raw(self) -> Vec<u8> {
        self.data
    }

    fn delta_encode(&self) -> Bytes {
        let mut encoded = BytesMut::with_capacity(self.data.len());

        // Store first byte as-is since we need a reference point
        if !self.data.is_empty() {
            encoded.put_u8(self.data[0]);

            // For each subsequent byte, store its difference from previous byte
            for i in 1..self.data.len() {
                encoded.put_u8(self.data[i].wrapping_sub(self.data[i - 1]));
            }
        }

        encoded.freeze()
    }

    fn delta_decode(data: &[u8]) -> Vec<u8> {
        let mut decoded = Vec::with_capacity(data.len());

        if !data.is_empty() {
            // First byte was stored as-is
            decoded.push(data[0]);
            let mut prev = data[0];

            // For each delta value, add it to previous byte to get original value
            for &delta in data.iter().skip(1) {
                let current = prev.wrapping_add(delta);
                decoded.push(current);
                prev = current;
            }
        }

        decoded
    }

    pub fn encode(&self) -> Result<Bytes, Error> {
        let mut compressor = Compressor::new(22)?;
        compressor.long_distance_matching(true)?;
        compressor.include_checksum(false)?;

        let compressed = compressor.compress(&self.delta_encode())?;

        let mut encoded = BytesMut::with_capacity(
            size_of::<Header>() + size_of::<InformationV1>() + compressed.len(),
        );

        // Write header
        encoded.put(Header::default().to_bytes());
        encoded.put(InformationV1::new(Flags::empty(), self.width, self.height).to_bytes());
        encoded.put(compressed.as_slice());

        Ok(encoded.freeze())
    }

    pub fn decode(mut data: &[u8]) -> Result<Self, Error> {
        let header = Header::from_bytes(data.copy_to_bytes(size_of::<Header>()))?;

        if !header.is_valid() {
            return Err(Error::MalformedHeader);
        }

        match header.version {
            1 => {
                let InformationV1 { width, height, .. } =
                    InformationV1::from_bytes(data.copy_to_bytes(size_of::<InformationV1>()))?;

                let data = Self::delta_decode(&decode_all(data.reader())?);

                Ok(Self {
                    width,
                    height,
                    data,
                })
            }
            _ => Err(Error::Unsupported),
        }
    }
}
