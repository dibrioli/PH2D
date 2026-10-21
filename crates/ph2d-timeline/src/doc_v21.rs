//! ⭐ **A migração v21 → v22** — a imagem CONGELADA dos bytes v21 (e v20, que são os mesmos bytes).
//!
//! A v22 apendou `TimelineDoc.textos` no FIM do documento; o postcard é posicional e um v21 acaba
//! antes dele. A conversão dá a tabela VAZIA, sem perda: nenhuma build v21 publicava campos de
//! texto (`CampoInfo::texto` nasceu com a v22).
//!
//! ⛔ Não se edita: um campo que mude aqui deixa de ler os ficheiros que existem. O gate é
//! `tests/it/canal_textos.rs` contra `tests/fixtures/timeline_doc_v21.bin`.

use super::*;

/// A versão que esta imagem lê.
pub(super) const VERSION: u32 = 21;

#[derive(Deserialize)]
struct TimelineDocV21 {
    version: u32,
    fps_display: f64,
    clips: Vec<NamedClip>,
    active_clip: usize,
    bindings: Vec<TargetBinding>,
    markers: Vec<Marker>,
    next_target: u64,
    next_strip: u64,
    stack: Vec<ClipLane>,
    containers: Vec<NamedContainer>,
    scene_length: Option<f64>,
}

/// Lê bytes v21 como um documento v22 sem textos.
pub(super) fn migrate(bytes: &[u8]) -> Result<TimelineDoc, String> {
    ler(bytes, VERSION)
}

/// Lê bytes desta imagem gravados com a versão `versao` (a v20 partilha-a).
pub(super) fn ler(bytes: &[u8], versao: u32) -> Result<TimelineDoc, String> {
    let old: TimelineDocV21 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    debug_assert_eq!(old.version, versao);
    Ok(TimelineDoc {
        version: DOC_VERSION,
        fps_display: old.fps_display,
        clips: old.clips,
        active_clip: old.active_clip,
        bindings: old.bindings,
        markers: old.markers,
        next_target: old.next_target,
        next_strip: old.next_strip,
        stack: old.stack,
        containers: old.containers,
        scene_length: old.scene_length,
        textos: Vec::new(),
        scratch: StackScratch::default(),
        reverse_play: false,
    })
}
