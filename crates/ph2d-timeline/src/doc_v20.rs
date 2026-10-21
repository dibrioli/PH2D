//! **A migração v20 → v22** — o `PropKind::Field` foi APENDADO (o postcard grava o índice da
//! variante), então os bytes de um v20 SÃO um v21 sem campos: lêem-se pela imagem v21
//! (`doc_v21.rs`), que só dá a tabela de textos vazia. Provada em `tests/it/canal_field.rs` contra
//! `tests/fixtures/timeline_doc_v20.bin`.

use super::*;

pub(super) const VERSION: u32 = 20;

pub(super) fn migrate(bytes: &[u8]) -> Result<TimelineDoc, String> {
    super::v21::ler(bytes, VERSION)
}
