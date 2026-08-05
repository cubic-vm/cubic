use crate::models::HashAlg;
use crate::util::hex_encode;
use sha2::{Digest, Sha256, Sha512};

pub enum Hasher {
    Sha256(Sha256),
    Sha512(Sha512),
}

impl Hasher {
    pub fn new(hash_alg: HashAlg) -> Self {
        match hash_alg {
            HashAlg::Sha256 => Hasher::Sha256(Sha256::new()),
            HashAlg::Sha512 => Hasher::Sha512(Sha512::new()),
        }
    }

    pub fn update(&mut self, buf: &[u8]) {
        match self {
            Hasher::Sha256(hasher) => hasher.update(buf),
            Hasher::Sha512(hasher) => hasher.update(buf),
        }
    }

    pub fn finalize(self) -> String {
        match self {
            Hasher::Sha256(hasher) => hex_encode(&hasher.finalize()),
            Hasher::Sha512(hasher) => hex_encode(&hasher.finalize()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_of_abc() {
        let mut hasher = Hasher::new(HashAlg::Sha256);
        hasher.update(b"abc");
        assert_eq!(
            hasher.finalize(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha512_of_abc() {
        let mut hasher = Hasher::new(HashAlg::Sha512);
        hasher.update(b"abc");
        assert_eq!(
            hasher.finalize(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn test_update_in_chunks_matches_single_update() {
        let mut chunked = Hasher::new(HashAlg::Sha256);
        chunked.update(b"a");
        chunked.update(b"bc");

        let mut single = Hasher::new(HashAlg::Sha256);
        single.update(b"abc");

        assert_eq!(chunked.finalize(), single.finalize());
    }
}
