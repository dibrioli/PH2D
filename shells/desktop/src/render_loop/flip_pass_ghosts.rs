//! **Ghost Frames** (ADR-0114 W3.T3.3) — os fantasmas dos desenhos vizinhos,
//! desenhados ANTES da arte do quadro atual.
//!
//! O QUEM é decidido pela função pura `ph2d_flip::ghosts` (port do `get_frame_id`,
//! testada headless nos 3 modos). Aqui só o COMO: cada fantasma é um passe do
//! MESMO rasterizador, com a câmera marcada por um `ghost_tint` — a arte sai como
//! **silhueta recolorida** (verde = passado, azul = futuro) e esmaecida pelo fade
//! `1/|Δ|`. Custo por fantasma: 1 upload + 1 draw, com a tesselação vinda do
//! **cache por desenho** (o mesmo do quadro atual — um ghost de um desenho já
//! visitado não re-empacota nada).
//!
//! Gates (todos do GP, `02_referencia §8`):
//! - **some no play** — durante a reprodução o fantasma é ruído puro;
//! - `onion.enabled` por objeto e `use_onion` por camada (o fundo não vira ghost);
//! - camada invisível não gera fantasma (não se vê o que não está lá).

use super::flip_pass_cache::TessCache;
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipObjectId, Frame};
use ph2d_flip_render::{CameraRaw, FlipRenderer};
use ph2d_gpu::GpuContext;
use ph2d_render::GameRt;
use ph2d_vec_scene::Xform;

/// Desenha os fantasmas de TODOS os objetos no `game_rt` (premult-over, por baixo
/// da arte do quadro — que compõe depois). `selected` são as chaves marcadas na
/// tira (só o modo `Selected` as usa). No-op durante o play.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    flip: &FlipDoc,
    flip_render: &mut FlipRenderer,
    tess: &mut TessCache,
    models: &[(FlipObjectId, Xform)],
    playhead: &Playhead,
    selected: &[Frame],
    game_rt: &GameRt,
    cam: &CameraRaw,
    size: (u32, u32),
    gpu: &GpuContext,
) {
    if playhead.is_playing() {
        return; // o fantasma some no play (regra de produto do GP)
    }
    for obj in flip.objects() {
        if !obj.onion.enabled {
            continue;
        }
        let frame = obj.frame_at(playhead);
        let model = models
            .iter()
            .find(|(id, _)| *id == obj.id)
            .map_or(Xform::IDENTITY, |(_, x)| *x);
        let base = if model.is_identity() {
            *cam
        } else {
            super::flip_pass::fold_model(cam, &model)
        };
        for layer in obj.layers() {
            if !layer.visible || !layer.use_onion {
                continue;
            }
            for g in ph2d_flip::ghosts(layer, frame, &obj.onion, selected) {
                let Some(art) = obj.drawing(g.drawing) else {
                    continue;
                };
                if art.strokes.is_empty() {
                    continue;
                }
                let key = (obj.id.0, g.drawing.0);
                tess.ensure(key, art);
                let Some(data) = tess.get(&key) else { continue };
                let ghost_cam = base.with_ghost_tint(
                    [g.tint.r(), g.tint.g(), g.tint.b()],
                    g.alpha * layer.opacity,
                );
                super::flip_pass::draw_overlay(flip_render, data, &ghost_cam, game_rt, size, gpu);
            }
        }
    }
}
