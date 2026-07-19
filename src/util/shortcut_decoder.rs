use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec::Decoder;

/// Whether the byte about to be decoded starts a fresh line, or a `~` was
/// already seen there and is waiting on the next byte to disambiguate.
enum State {
    Normal { at_line_start: bool },
    PendingTilde,
}

/// Decodes the OpenSSH-style detach sequence: Enter, then `~`, then `.`.
/// Bytes are emitted as soon as they are known not to be part of the
/// sequence. A `~` at the start of a line is withheld until the next byte
/// arrives. If that byte is `.`, decoding ends with an error, since the
/// `Decoder` trait has no other way to end a stream voluntarily while its
/// source stays open. Neither byte is ever emitted.
pub struct ShortcutDecoder {
    state: State,
}

impl ShortcutDecoder {
    pub fn new() -> Self {
        Self {
            state: State::Normal {
                at_line_start: true,
            },
        }
    }
}

impl Decoder for ShortcutDecoder {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> std::io::Result<Option<Bytes>> {
        let Some(&byte) = src.first() else {
            return Ok(None);
        };
        src.advance(1);

        match self.state {
            State::PendingTilde => {
                if byte == b'.' {
                    return Err(std::io::Error::other("shortcut sequence detected"));
                }
                self.state = State::Normal {
                    at_line_start: matches!(byte, b'\r' | b'\n'),
                };
                Ok(Some(Bytes::copy_from_slice(&[b'~', byte])))
            }
            State::Normal {
                at_line_start: true,
            } if byte == b'~' => {
                self.state = State::PendingTilde;
                // The disambiguating byte may already be buffered. Resolve
                // it now instead of waiting for another read from the
                // source, which could stall on data we already have.
                self.decode(src)
            }
            State::Normal { .. } => {
                self.state = State::Normal {
                    at_line_start: matches!(byte, b'\r' | b'\n'),
                };
                Ok(Some(Bytes::copy_from_slice(&[byte])))
            }
        }
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> std::io::Result<Option<Bytes>> {
        if let Some(item) = self.decode(src)? {
            return Ok(Some(item));
        }
        if matches!(self.state, State::PendingTilde) {
            self.state = State::Normal {
                at_line_start: false,
            };
            return Ok(Some(Bytes::from_static(b"~")));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};
    use tokio_util::codec::FramedRead;
    use tokio_util::io::StreamReader;

    fn shortcut_reader<I: tokio::io::AsyncRead>(
        inner: I,
    ) -> StreamReader<FramedRead<I, ShortcutDecoder>, Bytes> {
        StreamReader::new(FramedRead::new(inner, ShortcutDecoder::new()))
    }

    #[test]
    fn detach_on_tilde_dot_at_session_start() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"~."[..]);
        assert!(decoder.decode(&mut src).is_err());
    }

    #[test]
    fn detach_on_tilde_dot_after_newline() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"a\n"[..]);
        assert_eq!(
            Some(Bytes::from_static(b"a")),
            decoder.decode(&mut src).unwrap()
        );
        assert_eq!(
            Some(Bytes::from_static(b"\n")),
            decoder.decode(&mut src).unwrap()
        );

        let mut src = BytesMut::from(&b"~."[..]);
        assert!(decoder.decode(&mut src).is_err());
    }

    #[test]
    fn no_detach_when_tilde_is_mid_line() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"a~."[..]);
        assert_eq!(
            Some(Bytes::from_static(b"a")),
            decoder.decode(&mut src).unwrap()
        );
        assert_eq!(
            Some(Bytes::from_static(b"~")),
            decoder.decode(&mut src).unwrap()
        );
        assert_eq!(
            Some(Bytes::from_static(b".")),
            decoder.decode(&mut src).unwrap()
        );
    }

    #[test]
    fn replay_withheld_tilde_when_next_byte_is_not_a_dot() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"~x"[..]);
        assert_eq!(
            Some(Bytes::copy_from_slice(b"~x")),
            decoder.decode(&mut src).unwrap()
        );
    }

    #[test]
    fn replay_tilde_on_repeated_tilde() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"~~"[..]);
        assert_eq!(
            Some(Bytes::copy_from_slice(b"~~")),
            decoder.decode(&mut src).unwrap()
        );
    }

    #[test]
    fn waits_for_more_data_after_a_lone_tilde() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"~"[..]);
        assert_eq!(None, decoder.decode(&mut src).unwrap());
    }

    #[test]
    fn decode_eof_flushes_a_withheld_tilde() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::from(&b"~"[..]);
        assert_eq!(None, decoder.decode(&mut src).unwrap());
        assert_eq!(
            Some(Bytes::from_static(b"~")),
            decoder.decode_eof(&mut src).unwrap()
        );
        assert_eq!(None, decoder.decode_eof(&mut src).unwrap());
    }

    #[test]
    fn decode_eof_is_a_no_op_without_a_withheld_tilde() {
        let mut decoder = ShortcutDecoder::new();
        let mut src = BytesMut::new();
        assert_eq!(None, decoder.decode_eof(&mut src).unwrap());
    }

    #[tokio::test]
    async fn detach_on_enter_tilde_dot_suppresses_both_bytes() {
        let (mut writer, reader) = duplex(64);
        writer.write_all(b"~.").await.unwrap();
        let mut shortcut = shortcut_reader(reader);

        let mut buf = [0u8; 8];
        assert!(shortcut.read(&mut buf).await.is_err());
    }

    #[tokio::test]
    async fn deliver_withheld_tilde_when_stream_ends_without_a_dot() {
        let (mut writer, reader) = duplex(64);
        writer.write_all(b"~").await.unwrap();
        drop(writer);
        let mut shortcut = shortcut_reader(reader);

        let mut received = Vec::new();
        shortcut.read_to_end(&mut received).await.unwrap();
        assert_eq!(b"~".to_vec(), received);
    }

    #[tokio::test]
    async fn withholds_tilde_until_next_byte_arrives() {
        let (mut writer, reader) = duplex(64);
        writer.write_all(b"~").await.unwrap();
        let mut shortcut = shortcut_reader(reader);

        let mut buf = [0u8; 8];
        let read = tokio::time::timeout(Duration::from_millis(50), shortcut.read(&mut buf)).await;
        assert!(
            read.is_err(),
            "read should still be pending with only '~' available"
        );

        writer.write_all(b".").await.unwrap();
        assert!(shortcut.read(&mut buf).await.is_err());
    }

    #[tokio::test]
    async fn withheld_tilde_replays_once_a_non_dot_byte_arrives() {
        let (mut writer, reader) = duplex(64);
        writer.write_all(b"~").await.unwrap();
        let mut shortcut = shortcut_reader(reader);

        let mut buf = [0u8; 8];
        let read = tokio::time::timeout(Duration::from_millis(50), shortcut.read(&mut buf)).await;
        assert!(read.is_err());

        writer.write_all(b"x").await.unwrap();
        let size = shortcut.read(&mut buf).await.unwrap();
        assert_eq!(b"~x", &buf[..size]);
    }
}
