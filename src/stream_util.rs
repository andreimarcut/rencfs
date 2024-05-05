use std::cmp::min;
use std::io;
use std::io::{Read, Write};
use num_format::{Locale, ToFormattedString};
use tracing::{debug, error, instrument, warn};
use crate::encryptedfs::EncryptedFs;

#[cfg(test)]
const BUF_SIZE: usize = 256 * 1024;
// 256 KB buffer, smaller for tests because they all run in parallel
#[cfg(not(test))]
const BUF_SIZE: usize = 1024 * 1024; // 1 MB buffer

#[instrument(skip(r, len), fields(len = len.to_formatted_string( & Locale::en)))]
pub fn seek_forward(r: &mut impl Read, len: u64) -> io::Result<()> {
    debug!("");
    if len == 0 {
        return Ok(());
    }

    let mut buffer = vec![0; BUF_SIZE];
    let mut pos = 0_u64;
    loop {
        let read_len = if pos + buffer.len() as u64 > len {
            (len - pos) as usize
        } else {
            buffer.len()
        };
        // debug!(pos = pos.to_formatted_string(&Locale::en), read_len = read_len.to_formatted_string(&Locale::en), "reading");
        if read_len > 0 {
            let read = r.read(&mut buffer[..read_len]).map_err(|err| {
                error!("error reading from file pos {} len {}", pos.to_formatted_string(&Locale::en), read_len.to_formatted_string(&Locale::en));
                err
            })?;
            pos += read as u64;
            if pos == len {
                break;
            } else if read == 0 {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected eof"))?;
            }
        } else {
            break;
        }
    }

    Ok(())
}

#[instrument(skip(r, w, len), fields(len = len.to_formatted_string(& Locale::en)))]
pub fn copy_exact(r: &mut impl Read, w: &mut impl Write, len: u64) -> io::Result<()> {
    debug!("");
    if len == 0 {
        return Ok(());
    }
    let mut buffer = vec![0; BUF_SIZE];
    let mut read_pos = 0_u64;
    loop {
        let buf_len = min(buffer.len(), (len - read_pos) as usize);
        debug!("reading from file pos {} buf_len {}", read_pos.to_formatted_string(&Locale::en), buf_len.to_formatted_string(&Locale::en));
        r.read_exact(&mut buffer[..buf_len]).map_err(|err| {
            error!("error reading from file pos {} len {}",  read_pos.to_formatted_string(&Locale::en), buf_len.to_formatted_string(&Locale::en));
            err
        })?;
        w.write_all(&buffer[..buf_len])?;
        read_pos += buf_len as u64;
        if read_pos == len {
            break;
        }
    }
    Ok(())
}

#[instrument(skip(w, len), fields(len = len.to_formatted_string(& Locale::en)))]
pub fn fill_zeros(w: &mut impl Write, len: u64) -> io::Result<()> {
    debug!("");
    if len == 0 {
        return Ok(());
    }
    let buffer = vec![0; BUF_SIZE];
    let mut written = 0_u64;
    loop {
        let buf_len = min(buffer.len(), (len - written) as usize);
        w.write_all(&buffer[..buf_len])?;
        written += buf_len as u64;
        if written == len {
            break;
        }
    }
    Ok(())
}

pub async fn write_all_string_to_fs(fs: &EncryptedFs, ino: u64, offset: u64, s: &str, fh: u64) -> anyhow::Result<()> {
    write_all_bytes_to_fs(fs, ino, offset, s.as_bytes(), fh).await
}

pub async fn write_all_bytes_to_fs(fs: &EncryptedFs, ino: u64, offset: u64, buf: &[u8], fh: u64) -> anyhow::Result<()> {
    let mut pos = 0_usize;
    loop {
        let len = fs.write(ino, offset, &buf[pos..], fh).await.unwrap();
        pos += len;
        if pos == buf.len() {
            break;
        }
    }
    Ok(())
}