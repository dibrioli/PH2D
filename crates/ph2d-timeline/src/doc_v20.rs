//! **A migração v20 → v21** — o `PropKind::Field` foi APENDADO (o postcard grava o índice da
//! variante), então os bytes de um v20 SÃO um v21 sem campos: lê-se como o documento de agora e só
//! a versão muda. Provada em `tests/it/canal_field.rs` contra `tests/fixtures/timeline_doc_v20.bin`.

use super::*;

pub(super) const VERSION: u32 = 20;

pub(super) fn migrate(bytes: &[u8]) -> Result<TimelineDoc, String> {
    let mut doc: TimelineDoc = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    debug_assert_eq!(doc.version, VERSION);
    doc.version = DOC_VERSION;
    Ok(doc)
}
