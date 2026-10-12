//! A miniatura de um vídeo: do disco (a cache de quem já a viu) ou da rede, com prazo. Corre num fio de
//! quem chama — nunca no da janela.

use std::path::PathBuf;
use std::time::Duration;

use ph2d_board_model::Video;

/// O maior ficheiro de miniatura aceite: o `hqdefault.jpg` do YouTube tem dezenas de KB (medido no teste
/// `fetches_a_real_thumbnail`); um tecto de memória, não de produto.
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Porque não há miniatura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbError {
    /// Sem rede, o servidor recusou, ou passou o prazo.
    Network(String),
    /// Chegou algo que não é um JPEG.
    NotAnImage,
}

/// Os bytes JPEG da miniatura de `v` (o `hqdefault` do YouTube): primeiro a cache em disco, depois a rede
/// (com `timeout` para tudo), e o que chega guarda-se na cache.
///
/// # Errors
/// [`ThumbError`].
pub fn fetch(v: &Video, timeout: Duration) -> Result<Vec<u8>, ThumbError> {
    let file = cache_dir().map(|d| d.join(format!("{}.jpg", key(v))));
    if let Some(bytes) = file.as_ref().and_then(|f| std::fs::read(f).ok())
        && is_jpeg(&bytes)
    {
        return Ok(bytes);
    }
    let bytes = get(&ph2d_board_embed::thumbnail_url(v), timeout)?;
    if !is_jpeg(&bytes) {
        return Err(ThumbError::NotAnImage);
    }
    if let Some(f) = file {
        // Escrita atómica: um ficheiro a meio nunca é lido como miniatura.
        let tmp = f.with_extension("part");
        if f.parent().is_some_and(|p| std::fs::create_dir_all(p).is_ok())
            && std::fs::write(&tmp, &bytes).is_ok()
        {
            let _ = std::fs::rename(&tmp, &f);
        }
    }
    Ok(bytes)
}

fn key(v: &Video) -> String {
    match v {
        Video::YouTube { id, .. } => format!("youtube-{id}"),
    }
}

fn is_jpeg(b: &[u8]) -> bool {
    b.starts_with(&[0xFF, 0xD8, 0xFF])
}

/// `$XDG_CACHE_HOME/ph2d/video` (ou `~/.cache/…`).
fn cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("ph2d").join("video"))
}

/// Um GET por HTTPS com os certificados do sistema.
pub(crate) fn get(url: &str, timeout: Duration) -> Result<Vec<u8>, ThumbError> {
    use ureq::tls::{Certificate, RootCerts, TlsConfig, TlsProvider};
    let native = rustls_native_certs::load_native_certs();
    let roots: Vec<Certificate<'static>> = native
        .certs
        .iter()
        .map(|c| Certificate::from_der(c.as_ref()).to_owned())
        .collect();
    let tls = TlsConfig::builder()
        .provider(TlsProvider::Rustls)
        .root_certs(RootCerts::new_with_certs(&roots))
        .build();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(timeout))
        .build()
        .into();
    let mut resp = agent
        .get(url)
        .call()
        .map_err(|e| ThumbError::Network(e.to_string()))?;
    resp.body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|e| ThumbError::Network(e.to_string()))
}
