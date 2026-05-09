use anyhow::Result;
use quinn::Endpoint;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use quic_socks::certs::generate_self_signed_cert;
use tokio::io::copy_bidirectional;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:4433".parse::<SocketAddr>()?;
    
    let (cert_der, priv_key) = generate_self_signed_cert()?;
    let priv_key = rustls::pki_types::PrivatePkcs8KeyDer::from(priv_key);
    let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert_der)];

    let server_config = quinn::ServerConfig::with_single_cert(cert_chain, priv_key.into())?;
    let endpoint = Endpoint::server(server_config, addr)?;
    
    println!("🚀 QUIC Server on {}", addr);

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            let connection = conn.await.unwrap();
            while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                tokio::spawn(async move {
                    // 1. Читаем адрес из стрима (сериализация простая: len + string + u16)
                    let len = recv.read_u8().await.unwrap() as usize;
                    let mut host_buf = vec![0u8; len];
                    recv.read_exact(&mut host_buf).await.unwrap();
                    let host = String::from_utf8(host_buf).unwrap();
                    let port = recv.read_u16().await.unwrap();

                    println!("🔗 Connecting to {}:{}", host, port);

                    // 2. TCP коннект к целевому сайту
                    if let Ok(mut target) = TcpStream::connect(format!("{}:{}", host, port)).await {
                        let _ = copy_bidirectional(&mut target, &mut tokio::io::join(recv, send)).await;
                    }
                });
            }
        });
    }
    Ok(())
}

