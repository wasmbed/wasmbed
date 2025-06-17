use std::fs::File;
use std::io::Write;

const SERVER_PORT: u16 = 6443;
const SERVER_ADDRESS: &'static str = "127.0.0.1";

fn write_pem(path: &str, pem: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(pem.as_bytes()).unwrap();
}

#[cfg(test)]
mod client {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::task::JoinHandle;
    use wasmbed_gateway::kube_handler::KubeClient;
    use std::io::Error;
    use tokio::time::sleep;
    use tokio_rustls::TlsConnector;
    use std::io::Write;
    use tokio::net::TcpStream;
    use tokio_rustls::rustls;
    use tokio_rustls::client::TlsStream;
    use std::sync::Arc;
    use std::time::Duration;
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::ClientConfig;
    use wasmbed_gateway::simple_server::Server;
    use wasmbed_protocol::types::{
        ClientMessage, Envelope, Heartbeat, HeartbeatAcknowledge, Message,
        ServerMessage, Version,
    };

    const ERROR_CLIENT_DECODE_MESSAGE: &'static str =
        "Client Error decoding messages";

    async fn read_client_message(mut stream: TlsStream<TcpStream>) -> Envelope {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        let envelope = minicbor::decode::<Envelope>(&buffer[0..n])
            .map_err(|_| {
                Error::new(
                    std::io::ErrorKind::Other,
                    ERROR_CLIENT_DECODE_MESSAGE,
                )
            })
            .unwrap();

        envelope
    }

    // fn write_pem(path: &str, pem: &str) {
    //     let mut file = File::create(path).unwrap();
    //     file.write_all(pem.as_bytes()).unwrap();
    // }

    async fn start_server(
        port: u16,
        addr: &str,
    ) -> (Arc<Server>, JoinHandle<()>) {
        let kube_client = KubeClient::new().await.unwrap();
        let server = Arc::new(Server::new(addr.to_string(), port, kube_client));
        write_pem(CA_PATH, &server.state.test_pki.ca_cert.cert.pem());
        write_pem(
            CLIENT_CERT_PATH,
            &server.state.test_pki.client_cert.cert.pem(),
        );
        write_pem(
            CLIENT_KEY_PATH,
            &server.state.test_pki.client_cert.key_pair.serialize_pem(),
        );

        let inner_server = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            inner_server.run().await;
        });

        sleep(Duration::from_millis(5000)).await;

        (server, handle)
    }

    #[tokio::test]
    async fn test_client() {
        let (server, _handle) = start_server(SERVER_PORT, SERVER_ADDRESS).await;

        let client_cert =
            vec![CertificateDer::from_pem_file(CLIENT_CERT_PATH).unwrap()];

        let client_prv_key =
            PrivateKeyDer::from_pem_file(CLIENT_KEY_PATH).unwrap();

        let client_conf = ClientConfig::builder()
            .with_root_certificates(server.state.test_pki.roots.clone())
            .with_client_auth_cert(client_cert, client_prv_key)
            .unwrap();

        let domain = SERVER_ADDRESS.try_into().unwrap();
        let connector = TlsConnector::from(Arc::new(client_conf));

        // let mut conn = ClientConnection::new(Arc::new(client_conf), server_name).unwrap();
        let sock =
            TcpStream::connect(format!("{}:{}", SERVER_ADDRESS, SERVER_PORT))
                .await
                .unwrap();

        let heartbeat_message = Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::Heartbeat(
                Heartbeat::Heartbeat,
            )),
        };

        let encoded_message = minicbor::to_vec(heartbeat_message).unwrap();

        let mut tls_stream = connector.connect(domain, sock).await.unwrap();
        let _ = tls_stream.write_all(encoded_message.as_slice()).await;

        let heartbeat_reply = read_client_message(tls_stream).await;

        assert_eq!(
            Envelope {
                version: Version::V0,
                body: Message::ServerMessage(
                    ServerMessage::HeartbeatAcknowledge(
                        HeartbeatAcknowledge::HeartbeatAcknowledge,
                    )
                ),
            },
            heartbeat_reply
        );

        server.stop();
    }
}
