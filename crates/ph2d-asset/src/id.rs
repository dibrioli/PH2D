//! [`AssetId`] — **re-exportado** da crate-folha [`ph2d_asset_id`].
//!
//! ⚠️ **O tipo MUDOU DE CASA em 2026-08-27** (plano 33, W3) e este ficheiro ficou como a porta.
//! O motivo: o `Paint::Pattern` da `ph2d-vec-scene` guarda QUAL imagem um padrão usa, e aquela
//! crate é uma folha **pura** que não pode puxar os descodificadores de imagem que esta arrasta.
//!
//! ⭐ **Nada muda para quem chama:** `ph2d_asset::AssetId` e `ph2d_asset::id::AssetId` continuam a
//! resolver, e os 78 ficheiros que os usam não mudaram uma linha. Um segundo tipo de id para a
//! mesma coisa seria duas respostas à mesma pergunta.
pub use ph2d_asset_id::AssetId;
