//! ⭐ **A migração v18 → v19** — a imagem CONGELADA dos bytes v18, e a única coisa que a lê.
//!
//! Postcard é posicional: o v19 acrescentou `NamedClip.owner` no FIM de cada clip, que fica no
//! MEIO do documento, então os bytes v18 não se lêem como v19 com um campo em falta — lêem-se
//! deslocados. Estas duas estruturas são o v18 campo a campo (os tipos que não mudaram são os
//! vivos), e a conversão põe todo clip na Cena: o documento global de antes É a Animação da
//! Cena (doc ObjectsAndComponents 01 §6, zero perda).
//!
//! ⛔ Não se edita: um campo que mude aqui deixa de ler os ficheiros que existem. O gate é
//! `tests/it/clip_owner.rs` contra `tests/fixtures/timeline_doc_v18.bin`.

use super::*;

/// A versão que esta imagem lê.
pub(super) const VERSION: u32 = 18;

#[derive(Deserialize)]
struct NamedClipV18 {
    name: String,
    clip: Clip,
    loop_range: Option<(f64, f64)>,
    loop_ping_pong: bool,
    keys_loop_range: Option<(f64, f64)>,
    keys_loop_ping_pong: bool,
    length_override: Option<f64>,
    expr: BTreeMap<AnimTarget, String>,
    paths: BTreeMap<AnimTarget, crate::path::MotionPath>,
}

#[derive(Deserialize)]
struct TimelineDocV18 {
    version: u32,
    fps_display: f64,
    clips: Vec<NamedClipV18>,
    active_clip: usize,
    bindings: Vec<TargetBinding>,
    markers: Vec<Marker>,
    next_target: u64,
    next_strip: u64,
    stack: Vec<ClipLane>,
    containers: Vec<NamedContainer>,
    scene_length: Option<f64>,
}

/// Lê bytes v18 como um documento v19 com todos os clips na Cena.
pub(super) fn migrate(bytes: &[u8]) -> Result<TimelineDoc, String> {
    let old: TimelineDocV18 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
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
                owner: None,
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
        textos: Vec::new(),
        scratch: StackScratch::default(),
        reverse_play: false,
    })
}
