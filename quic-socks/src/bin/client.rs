use anyhow::Result;
use quinn::{Endpoint, Connection};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, copy_bidirectional};
use quic_socks::Socks5Header;

#[tokio::main]
async fn main() -> Result<()> {
    let local_proxy_addr = "127.0.0.1:1080".parse::<SocketAddr>()?;
    let remote_server_addr = "127.0.0.1:4433".parse::<SocketAddr>()?;

    // 1. Настройка QUIC клиента (пропускаем проверку TLS для теста)
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    // Подключаемся к нашему QUIC серверу
    let conn = endpoint.connect(remote_server_addr, "localhost")?.await?;
    println!("✅ Connected to QUIC server");

    // 2. Запускаем локальный TCP сервер (SOCKS5 вход)
    let listener = TcpListener::bind(local_proxy_addr).await?;
    println!("🎧 SOCKS5 proxy ready on {}", local_proxy_addr);

    while let Ok((mut tcp_stream, _)) = listener.accept().await {
        let quic_conn = conn.clone();
        tokio::spawn(async move {
            // А) Рукопожатие SOCKS5
            if let Ok(header) = Socks5Header::handshake(&mut tcp_stream).await {
                println!("🔌 Request to {}:{}", header.host, header.port);

                // Б) Открываем стрим внутри QUIC
                if let Ok((mut send, mut recv)) = quic_conn.open_bi().await {
                    // В) Передаем серверу адрес назначения (простой протокол)
                    let host_bytes = header.host.as_bytes();
                    let _ = send.write_u8(host_bytes.len() as u8).await;
                    let _ = send.write_all(host_bytes).await;
                    let _ = send.write_u16(header.port).await;

                    // Г) Соединяем потоки данных
                    let _ = copy_bidirectional(&mut tcp_stream, &mut tokio::io::join(recv, send)).await;
                }
            }
        });
    }
    Ok(())
}

// Утилита для игнорирования проверки сертификата (только для тестов!)
struct SkipServerVerification;
impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(&self, _: &rustls::pki_types::CertificateDer, _: &[rustls::pki_types::CertificateDer], _: &rustls::pki_types::ServerName, _: &[u8], _: rustls::pki_types::UnixTime) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _: &[u8], _: &rustls::pki_types::CertificateDer, _: &rustls::client::danger::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(&self, _: &[u8], _: &rustls::pki_types::CertificateDer, _: &rustls::client::danger::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider().signature_verification_algorithms.supported_schemes()
    }
}

