//! ⭐ **A migração v19 → v20** — a imagem CONGELADA dos bytes v19, e a única coisa que a lê.
//!
//! Postcard é posicional: a v20 acrescentou `NamedClip.autoplay` no FIM de cada clip, que fica no
//! MEIO do documento. Estas duas estruturas são o v19 campo a campo (os tipos que não mudaram são
//! os vivos), e a conversão não marca entrada nenhuma: a de cada dono é o 1.º clipe dele
//! ([`TimelineDoc::entry_clip_of`]), zero perda.
//!
//! ⛔ Não se edita: um campo que mude aqui deixa de ler os ficheiros que existem. O gate é
//! `tests/it/clip_entry.rs` contra `tests/fixtures/timeline_doc_v19.bin`.

use super::*;

/// A versão que esta imagem lê.
pub(super) const VERSION: u32 = 19;

#[derive(Deserialize)]
struct NamedClipV19 {
    name: String,
    clip: Clip,
    loop_range: Option<(f64, f64)>,
    loop_ping_pong: bool,
    keys_loop_range: Option<(f64, f64)>,
    keys_loop_ping_pong: bool,
    length_override: Option<f64>,
    expr: BTreeMap<AnimTarget, String>,
    paths: BTreeMap<AnimTarget, crate::path::MotionPath>,
    owner: Option<ClipOwner>,
}

#[derive(Deserialize)]
struct TimelineDocV19 {
    version: u32,
    fps_display: f64,
    clips: Vec<NamedClipV19>,
    active_clip: usize,
    bindings: Vec<TargetBinding>,
    markers: Vec<Marker>,
    next_target: u64,
    next_strip: u64,
    stack: Vec<ClipLane>,
    containers: Vec<NamedContainer>,
    scene_length: Option<f64>,
}

/// Lê bytes v19 como um documento v20 sem entrada marcada.
pub(super) fn migrate(bytes: &[u8]) -> Result<TimelineDoc, String> {
    let old: TimelineDocV19 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    debug_assert_eq!(old.version, VERSION);
    Ok(TimelineDoc {
        version: DOC_VERSION,
        fps_display: old.fps_display,
        clips: old
            .clips
            .into_iter()
            .map(|c| NamedClip {
                name: c.name,
                clip: c.clip,
                loop_range: c.loop_range,
                loop_ping_pong: c.loop_ping_pong,
                keys_loop_range: c.keys_loop_range,
                keys_loop_ping_pong: c.keys_loop_ping_pong,
                length_override: c.length_override,
                expr: c.expr,
                paths: c.paths,
                owner: c.owner,
                autoplay: false,
            })
            .collect(),
        active_clip: old.active_clip,
        bindings: old.bindings,
        markers: old.markers,
        next_target: old.next_target,
        next_strip: old.next_strip,
        stack: old.stack,
        containers: old.containers,
        scene_length: old.scene_length,
        scratch: StackScratch::default(),
        reverse_play: false,
    })
}
