// We need the pinned version of the GenericArray used by all the cryptographic crates
// of the [Rust Crypto](https://github.com/RustCrypto) team.
use crypto_common::generic_array::GenericArray;
use sha1::{Digest, Sha1};
use sha2::Sha256;

use std::io::{self, Read};

fn hash_stream<T: Read, D: Digest>(
    mut stream: T,
    mut hasher: D,
) -> io::Result<GenericArray<u8, D::OutputSize>> {
    loop {
        let mut buf = [0; 1024];
        let size = stream.read(&mut buf)?;

        hasher.update(&buf[0..size]);

        if size < 1024 {
            break;
        }
    }

    Ok(hasher.finalize())
}

pub fn hash_stream_into_sha1<T: Read>(stream: T) -> io::Result<[u8; 20]> {
    let mut hasher = Sha1::new();

    Ok(hash_stream(stream, hasher)?.into())
}

pub fn hash_stream_into_sha256<T: Read>(stream: T) -> io::Result<[u8; 32]> {
    let mut hasher = Sha256::new();

    Ok(hash_stream(stream, hasher)?.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const SMALL: &[u8] = &[0, 1, 2];
    const EXACT_1024: &[u8] = &[15; 1024];
    const MULTIPLE_1024: &[u8] = &[15; 3 * 1024];
    const BIG: &[u8] = &[15; 1060];

    #[test]
    fn sha1_smaller_than_1024() {
        assert_eq!(
            hash_stream_into_sha1(SMALL).unwrap(),
            hex!("0c7a623fd2bbc05b06423be359e4021d36e721ad")
        )
    }

    #[test]
    fn sha1_exact_1024() {
        assert_eq!(
            hash_stream_into_sha1(EXACT_1024).unwrap(),
            hex!("2b29815b5d9004992939fcd8c53c572a345f8746")
        )
    }

    #[test]
    fn sha1_multiple_of_1024() {
        assert_eq!(
            hash_stream_into_sha1(MULTIPLE_1024).unwrap(),
            hex!("f070063c18c1cff9e1798ca39666a11bdd5a9e3a")
        )
    }

    #[test]
    fn sha1_bigger_than_1024() {
        assert_eq!(
            hash_stream_into_sha1(BIG).unwrap(),
            hex!("37e07383b248f79f784921205c575e7d523e08f3")
        )
    }

    #[test]
    fn sha256_smaller_than_1024() {
        assert_eq!(
            hash_stream_into_sha256(SMALL).unwrap(),
            hex!("ae4b3280e56e2faf83f414a6e3dabe9d5fbe18976544c05fed121accb85b53fc")
        )
    }

    #[test]
    fn sha256_exact_1024() {
        assert_eq!(
            hash_stream_into_sha256(EXACT_1024).unwrap(),
            hex!("b3bc03eb368b99608517d7d5d4a6b29e76a87df726222ce44d44e8a1d9b32181")
        )
    }

    #[test]
    fn sha256_multiple_of_1024() {
        assert_eq!(
            hash_stream_into_sha256(MULTIPLE_1024).unwrap(),
            hex!("94479210814727ebec13dcd4c386682a42e324a6e7e0753e84a0c9612ea912ee")
        )
    }

    #[test]
    fn sha256_bigger_than_1024() {
        assert_eq!(
            hash_stream_into_sha256(BIG).unwrap(),
            hex!("2d7c672ed2e4855e2c8cc7c0d95492f869799f4bd6151ff15690c8bb409b3a7b")
        )
    }
}
