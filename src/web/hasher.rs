use crate::models::HashAlg;
use crate::util::hex_encode;
use sha2::digest::DynDigest;
use sha2::{Digest, Sha256, Sha512};
use std::io::{self, Write};

pub struct Hasher {
    digest: Box<dyn DynDigest>,
}

impl Hasher {
    pub fn new(hash_alg: HashAlg) -> Self {
        let digest: Box<dyn DynDigest> = match hash_alg {
            HashAlg::Sha256 => Box::new(Sha256::new()),
            HashAlg::Sha512 => Box::new(Sha512::new()),
        };
        Self { digest }
    }

    pub fn finalize(self) -> String {
        hex_encode(&self.digest.finalize())
    }
}

impl Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_of_abc() {
        let mut hasher = Hasher::new(HashAlg::Sha256);
        hasher.write_all(b"abc").unwrap();
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(hasher.finalize(), expected);
    }

    #[test]
    fn test_sha512_of_abc() {
        let mut hasher = Hasher::new(HashAlg::Sha512);
        hasher.write_all(b"abc").unwrap();
        assert_eq!(
            hasher.finalize(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn test_write_in_chunks_matches_single_write() {
        let mut chunked = Hasher::new(HashAlg::Sha256);
        chunked.write_all(b"a").unwrap();
        chunked.write_all(b"bc").unwrap();

        let mut single = Hasher::new(HashAlg::Sha256);
        single.write_all(b"abc").unwrap();

        assert_eq!(chunked.finalize(), single.finalize());
    }
}
