//! O TLS que o QUIC exige é só o ENVELOPE: certificado auto-assinado por sessão e o convidado aceita
//! qualquer um. Quem autentica é o SPAKE2 com a chave do convite, por cima (`ph2d-board-net`), e a cifra
//! ponta a ponta dele é a mesma da rede local.

use std::io;
use std::sync::Arc;

use quinn_proto::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn_proto::rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use quinn_proto::rustls::crypto::{
    CryptoProvider, ring, verify_tls12_signature, verify_tls13_signature,
};
use quinn_proto::rustls::pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime,
};
use quinn_proto::rustls::{self, DigitallySignedStruct, SignatureScheme, version};
use quinn_proto::{ClientConfig, ServerConfig, TransportConfig};

pub(crate) const ALPN: &[u8] = b"ph2d-board/1";
/// O nome no certificado e no pedido do convidado (nunca verificado).
pub(crate) const NAME: &str = "ph2d-board";

fn other(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(ring::default_provider())
}

pub(crate) fn server(transport: &Arc<TransportConfig>) -> io::Result<ServerConfig> {
    let ck = rcgen::generate_simple_self_signed(vec![NAME.to_owned()]).map_err(other)?;
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(ck.signing_key.serialize_der()));
    let mut tls = rustls::ServerConfig::builder_with_provider(provider())
        .with_protocol_versions(&[&version::TLS13])
        .map_err(other)?
        .with_no_client_auth()
        .with_single_cert(vec![ck.cert.der().clone()], key)
        .map_err(other)?;
    tls.alpn_protocols = vec![ALPN.to_vec()];
    let quic = QuicServerConfig::try_from(tls).map_err(other)?;
    let mut cfg = ServerConfig::with_crypto(Arc::new(quic));
    cfg.transport_config(Arc::clone(transport));
    Ok(cfg)
}

pub(crate) fn client(transport: &Arc<TransportConfig>) -> io::Result<ClientConfig> {
    let p = provider();
    let mut tls = rustls::ClientConfig::builder_with_provider(Arc::clone(&p))
        .with_protocol_versions(&[&version::TLS13])
        .map_err(other)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AnyCert(p)))
        .with_no_client_auth();
    tls.alpn_protocols = vec![ALPN.to_vec()];
    let quic = QuicClientConfig::try_from(tls).map_err(other)?;
    let mut cfg = ClientConfig::new(Arc::new(quic));
    cfg.transport_config(Arc::clone(transport));
    Ok(cfg)
}

/// Aceita qualquer certificado, mas confere a assinatura do aperto do TLS (a chave é a do certificado
/// mostrado): o envelope fica íntegro; a identidade é do SPAKE2.
#[derive(Debug)]
struct AnyCert(Arc<CryptoProvider>);

impl ServerCertVerifier for AnyCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
