use crate::error::{Error, Result};
use crate::view::TransferView;
use russh::Channel;
use std::cmp::min;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BUF_SIZE: usize = 64 * 1024;

/// Read a single SCP protocol ack byte from the stream.
async fn read_ack(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<()> {
    let mut byte = [0u8; 1];
    stream.read_exact(&mut byte).await?;
    match byte[0] {
        0 => Ok(()),
        1 | 2 => {
            let mut msg = Vec::new();
            let mut c = [0u8; 1];
            loop {
                stream.read_exact(&mut c).await?;
                if c[0] == b'\n' {
                    break;
                }
                msg.push(c[0]);
            }
            Err(Error::Scp(String::from_utf8_lossy(&msg).to_string()))
        }
        _ => Err(Error::Scp(format!(
            "Unexpected SCP response byte: {}",
            byte[0]
        ))),
    }
}

fn display_name(name: &str) -> String {
    let start = name.len().saturating_sub(30);
    format!("{:30}", &name[start..])
}

/// Upload a local file to a remote host via the SCP protocol.
///
/// Uses a raw SSH exec channel with `scp -t` instead of SFTP, avoiding the
/// per-write ACK bottleneck in russh-sftp that limits throughput to ~2 MB/s.
pub async fn upload(
    channel: Channel<russh::client::Msg>,
    local_path: &Path,
    remote_path: &str,
) -> Result<()> {
    let metadata = tokio::fs::metadata(local_path).await?;
    let file_size = metadata.len();
    let file_name = local_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::InvalidPath(local_path.display().to_string()))?;

    channel
        .exec(true, format!("scp -t {remote_path}"))
        .await
        .map_err(|e| Error::Scp(format!("Failed to start remote scp: {e}")))?;

    let mut stream = channel.into_stream();
    read_ack(&mut stream).await?;

    let header = format!("C0644 {file_size} {file_name}\n");
    stream.write_all(header.as_bytes()).await?;
    read_ack(&mut stream).await?;

    let mut file = tokio::fs::File::open(local_path).await?;
    let mut buf = vec![0u8; BUF_SIZE];
    let mut transferred: u64 = 0;
    let mut view = TransferView::new(&display_name(file_name));

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        stream.write_all(&buf[..n]).await?;
        transferred += n as u64;
        view.update(transferred, Some(file_size));
    }

    stream.write_all(&[0]).await?;
    read_ack(&mut stream).await?;

    Ok(())
}

/// Download a remote file to the local host via the SCP protocol.
///
/// Returns `Err` if the remote path is a directory or does not exist,
/// allowing the caller to fall back to SFTP.
pub async fn download(
    channel: Channel<russh::client::Msg>,
    remote_path: &str,
    local_path: &Path,
) -> Result<()> {
    channel
        .exec(true, format!("scp -f {remote_path}"))
        .await
        .map_err(|e| Error::Scp(format!("Failed to start remote scp: {e}")))?;

    let mut stream = channel.into_stream();
    stream.write_all(&[0]).await?;

    let mut header_bytes = Vec::new();
    loop {
        let mut c = [0u8; 1];
        stream.read_exact(&mut c).await?;
        if c[0] == b'\n' {
            break;
        }
        header_bytes.push(c[0]);
    }

    if let Some(&first) = header_bytes.first()
        && (first == 1 || first == 2)
    {
        return Err(Error::Scp(
            String::from_utf8_lossy(&header_bytes[1..]).to_string(),
        ));
    }

    let header = String::from_utf8_lossy(&header_bytes);
    let parts: Vec<&str> = header.splitn(3, ' ').collect();
    if parts.len() != 3 || !parts[0].starts_with('C') {
        return Err(Error::Scp(format!("Invalid SCP header: {header}")));
    }
    let file_size: u64 = parts[1]
        .parse()
        .map_err(|_| Error::Scp(format!("Invalid file size in header: {}", parts[1])))?;
    let file_name = parts[2];

    stream.write_all(&[0]).await?;

    let target_path = if local_path.is_dir() {
        local_path.join(file_name)
    } else {
        local_path.to_path_buf()
    };

    let mut file = tokio::fs::File::create(&target_path).await?;
    let mut buf = vec![0u8; BUF_SIZE];
    let mut remaining = file_size;
    let mut view = TransferView::new(&display_name(file_name));

    while remaining > 0 {
        let to_read = min(remaining as usize, BUF_SIZE);
        let n = stream.read(&mut buf[..to_read]).await?;
        if n == 0 {
            return Err(Error::Scp("Unexpected end of stream".to_string()));
        }
        file.write_all(&buf[..n]).await?;
        remaining -= n as u64;
        view.update(file_size - remaining, Some(file_size));
    }

    let mut marker = [0u8; 1];
    stream.read_exact(&mut marker).await?;

    stream.write_all(&[0]).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_short() {
        assert_eq!(display_name("file.txt"), "file.txt                      ");
    }

    #[test]
    fn test_display_name_exact() {
        let name = "abcdefghijklmnopqrstuvwxyz1234";
        assert_eq!(name.len(), 30);
        assert_eq!(display_name(name), name);
    }

    #[test]
    fn test_display_name_long() {
        let name = "/very/long/path/to/some/deeply/nested/file.tar.gz";
        let result = display_name(name);
        assert_eq!(result.len(), 30);
        assert!(result.ends_with("file.tar.gz"));
    }
}
