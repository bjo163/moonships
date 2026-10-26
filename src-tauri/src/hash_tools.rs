use sha2::{Digest, Sha256};
use std::io::{self, Read};

pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const HASH_CHUNK_BYTES: usize = 64 * 1024;

pub fn sha256_text(text: &str) -> Result<String, &'static str> {
    if text.len() > MAX_TEXT_BYTES {
        return Err("text_too_large");
    }
    Ok(format_digest(Sha256::digest(text.as_bytes())))
}

pub fn sha256_reader<R: Read>(mut reader: R, max_bytes: u64) -> io::Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; HASH_CHUNK_BYTES];
    let mut total = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| io::Error::other("input size overflow"))?;
        if total > max_bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "file_too_large"));
        }
        hasher.update(&buffer[..read]);
    }
    Ok((format_digest(hasher.finalize()), total))
}

fn format_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{sha256_reader, sha256_text, HASH_CHUNK_BYTES};
    use sha2::Digest;
    use std::io::Cursor;

    #[test]
    fn sha256_known_vectors_match() {
        assert_eq!(
            sha256_text("").unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_text("abc").unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn reader_hashes_across_chunks_and_reports_byte_count() {
        let bytes = vec![b'a'; HASH_CHUNK_BYTES * 2 + 37];
        let (digest, count) =
            sha256_reader(Cursor::new(bytes.clone()), bytes.len() as u64).unwrap();
        assert_eq!(count, bytes.len() as u64);
        assert_eq!(digest, format!("{:x}", sha2::Sha256::digest(&bytes)));
    }

    #[test]
    fn reader_rejects_above_limit_without_returning_partial_hash() {
        assert_eq!(
            sha256_reader(Cursor::new(b"four"), 3)
                .unwrap_err()
                .to_string(),
            "file_too_large"
        );
    }
}
