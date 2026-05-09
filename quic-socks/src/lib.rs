pub mod certs;

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod certs {
    use rcgen::generate_simple_self_signed;
    use anyhow::Result;

    pub fn generate_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>)> {
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let cert = generate_simple_self_signed(subject_alt_names)?;
        Ok((cert.serialize_der()?, cert.serialize_private_key_der()))
    }
}

pub struct Socks5Header {
    pub host: String,
    pub port: u16,
}

impl Socks5Header {
    /// Выполняет рукопожатие SOCKS5 и возвращает целевой адрес
    pub async fn handshake<S>(mut stream: S) -> Result<Self> 
    where S: AsyncReadExt + AsyncWriteExt + Unpin 
    {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await?;
        
        if buf[0] != 0x05 {
            return Err(anyhow!("Not SOCKS5"));
        }

        // Поддерживаем только "No Authentication"
        let methods_len = buf[1] as usize;
        let mut methods = vec![0u8; methods_len];
        stream.read_exact(&mut methods).await?;
        stream.write_all(&[0x05, 0x00]).await?;

        // Читаем запрос на соединение
        let mut header = [0u8; 4];
        stream.read_exact(&mut header).await?;
        
        if header[1] != 0x01 {
            return Err(anyhow!("Only CONNECT command supported"));
        }

        let host = match header[3] {
            0x01 => { // IPv4
                let mut ip = [0u8; 4];
                stream.read_exact(&mut ip).await?;
                std::net::Ipv4Addr::from(ip).to_string()
            }
            0x03 => { // Domain name
                let len = stream.read_u8().await? as usize;
                let mut domain = vec![0u8; len];
                stream.read_exact(&mut domain).await?;
                String::from_utf8(domain)?
            }
            _ => return Err(anyhow!("Unsupported address type")),
        };

        let port = stream.read_u16().await?;
        
        // Отвечаем успехом (заглушка, реальный успех пошлем позже)
        stream.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;

        Ok(Socks5Header { host, port })
    }
}

