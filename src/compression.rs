use std::io::prelude::*;

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

/// Encodes given bytes using the gzip format.
pub fn gz_encode(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&bytes[..])?;
    Ok(e.finish()?)
}

/// Decodes given bytes using the gzip format.
pub fn gz_decode(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut result = Vec::new();
    gz.read_to_end(&mut result)?;
    Ok(result)
}

#[cfg(test)]
mod test {

    use crate::compression::*;

    #[test]
    fn test_hello() {
        let input = b"hello world".to_vec();
        let encoded = gz_encode(&input).unwrap();
        let decoded = gz_decode(&encoded).unwrap();
        assert_eq!(input, decoded.as_slice())
    }

    #[test]
    fn test_large() {
        let input = b"1234567890".repeat(1000).to_vec();
        let encoded = gz_encode(&input).unwrap();
        let decoded = gz_decode(&encoded).unwrap();
        assert_eq!(input, decoded.as_slice())
    }
}
