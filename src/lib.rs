use bytes::{BufMut, Bytes, BytesMut};
use zstd::{bulk::Compressor, decode_all};

// - "BOOP" magic number (4 bytes)
// - Version number (1 byte)
// - Width (4 bytes, little endian)
// - Height (4 bytes, little endian)
// - Compressed data length (4 bytes, little endian)
// - Compressed image data (variable length)

const MAGIC: &[u8] = b"BOOP";
const VERSION: u8 = 1;
const HEADER_SIZE: usize = MAGIC.len() + 1 + 4 + 4 + 4;

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

        let mut encoded = BytesMut::with_capacity(HEADER_SIZE + compressed.len());

        // Write header
        encoded.put(MAGIC);
        encoded.put_u8(VERSION);
        encoded.put_u32_le(self.width);
        encoded.put_u32_le(self.height);
        encoded.put_u32_le(compressed.len() as u32); // Compressed length

        // Write compressed data
        encoded.put(compressed.as_slice());

        Ok(encoded.freeze())
    }

    pub fn decode(data: &[u8]) -> Result<Self, Error> {
        // Validate minimum length and magic number
        if data.len() < HEADER_SIZE || &data[0..4] != MAGIC {
            return Err(Error::MalformedHeader);
        }

        let version = data[4];
        if version != VERSION {
            return Err(Error::Unsupported);
        }

        // FIXME: don't unwrap
        let width = u32::from_le_bytes(data[5..9].try_into().unwrap());
        let height = u32::from_le_bytes(data[9..13].try_into().unwrap());
        let compressed_len = u32::from_le_bytes(data[13..HEADER_SIZE].try_into().unwrap()) as usize;

        // Validate compressed data length
        if data.len() < HEADER_SIZE + compressed_len {
            return Err(Error::MalformedBody);
        }

        let data = Self::delta_decode(&decode_all(
            &data[HEADER_SIZE..HEADER_SIZE + compressed_len],
        )?);

        Ok(Self {
            width,
            height,
            data,
        })
    }
}
