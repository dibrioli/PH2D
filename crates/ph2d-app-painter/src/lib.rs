//! **A família PAINTER da shell** — a metade de *composição* do Painter (W2 Fase D).
//!
//! ⚠️ **Isto NÃO é o motor de pintura.** O motor vive na [`ph2d-tool-painter`] (136 093 LOC, a
//! maior crate do repo) e no [`ph2d-paint-gpu`]; o que mora aqui é o que estava dentro da
//! `shells/desktop` — as cenas de smoke, as pontes de desenho do `render_loop` e os corpos dos
//! gestos de canvas. *Código de FAMÍLIA vive em `crates/ph2d-app-<família>`; a shell é
//! COMPOSIÇÃO* (`CLAUDE.md` §2, `HOWTO_partir_uma_familia_da_shell.md`).
//!
//! # ⛔ O que ficou na shell, e é DESENHO
//!
//! O **LAÇO** (`render_loop/mod.rs`) garante a ordem do quadro e não se abstrai (HOWTO §4): o que
//! sai são os **CORPOS**, e a shell chama `crate::<mod>::<fn>` no ponto certo. Os
//! gestos de canvas ficam com **invólucros** `&mut App` na shell, porque os corpos deles precisam
//! do `gfx` e ⛔ **nenhuma porta do [`ph2d_app_host::AppHost`] devolve um handle** — é essa
//! proibição que segura a fronteira inteira.
//!
//! # ⚠️ A FACHADA que a régua do fecho contava como shell
//!
//! Estes seis roteadores pareciam presos à `shells/desktop` por `crate::image_import`. Aquele
//! ficheiro tem **6 linhas** e é `pub(crate) use ph2d_image_import::*` — *uma crate a usar o nome
//! da shell*. A `line/app-vec` mediu a mesma coisa na Fase C (cinco das seis âncoras dela eram
//! fachadas) e a leitura é a mesma: **a régua erra aqui no sentido conservador**, que é o oposto do
//! habitual, e por isso passa despercebida.

// ─────────────────────────────────────────────────────────────────────────
// **Os SEIS roteadores de cena.** Cada um lê a PRÓPRIA variável de ambiente
// (`enabled()`), o que é a condição do «fim da linha» desta wave — a shell
// só os chama.
// ─────────────────────────────────────────────────────────────────────────
pub mod impasto_smoke;
pub mod line_smoke;
pub mod mask_smoke;
pub mod shape_grab;

// ─────────────────────────────────────────────────────────────────────────
// **As PONTES DE DESENHO e o carimbo**, vindos de `shells/desktop/src/render_loop/`.
// ⚠️ Os nomes MANTÊM o prefixo `painter_` de propósito: um rename por nome corrompe a
// prosa que cita o ficheiro, e a Fase A da `line/app-physics` pagou 76 citações numa
// varredura só. Aqui dentro tudo é a família; o prefixo é só história que não custa nada.
// ─────────────────────────────────────────────────────────────────────────
/// `PH2D_PAINT_PERF` aggregation (one summary line per window, not per frame).
///
/// `pub(crate)` porque a frente L do plano 26 carimba a chegada do evento de ponteiro lá do
/// `input_dispatch` (a latência começa na ENTREGA, não no frame).
pub mod paint_perf;
pub mod painter_bridge;
/// Brush-image import helpers (Grain/Shape file pickers), split from
/// `painter_bridge` for the HR-18 file-LOC cap.
pub mod painter_bridge_assets;
/// The brush-cursor ring, split from `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub mod painter_bridge_brush_ring;
/// The Curve / Free Hand editor overlay (spine + control dots + tangent handles), split from
/// `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub mod painter_bridge_curve_overlay;
/// A pergunta *«esta entidade está na cena?»* que o extract faz — ver o módulo.
/// O `OnScreenEnabler` a decidir alguma coisa: *«só corre/aparece quando está no ecrã»*.
/// The Deform Transform gizmo (whole-region bounding box), split from `painter_bridge_overlays` (Wave 2).
pub mod painter_bridge_deform_gizmo;
/// The Fill (Bucket) ColorDrop cursor swatch overlay, split from `painter_bridge_overlays` for the
/// HR-18 file-LOC cap.
pub mod painter_bridge_fill_overlay;
/// Shared Sprite-style gizmo painting for the Curve + Stencil transform gizmos (theme tokens, darker).
pub mod painter_bridge_gizmo;
/// A rede do Grid Stamp desenhada sobre a sprite (o método carimba no centro da célula dela).
pub mod painter_bridge_grid;
/// The Line polyline editor overlay (segments + corner dots + transform gizmo + Fillet/Chamfer handles),
/// split from `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub mod painter_bridge_line_overlay;
/// Multi-shape op badges (`+`/`−`/`○` type-square glyph per shape + a frame for parked shapes).
pub mod painter_bridge_op_badges;
/// On-canvas editing chrome (brush ring + Curve/Circle/Polygon/Stencil overlays), split from
/// `painter_bridge` for the HR-18 file-LOC cap.
pub mod painter_bridge_overlays;
pub mod painter_bridge_phases;
pub mod painter_bridge_queries;
/// The isolated selection gizmos (ellipse / polygon / freehand), split from `painter_bridge_overlays`.
pub mod painter_bridge_selection_gizmos;
/// The Selection overlay (marching ants + deselected-area hatching), split from `painter_bridge_overlays`
/// for the HR-18 file-LOC cap.
pub mod painter_bridge_selection_overlay;
/// Live GPU preview of a brush Shape-source sprite (when not selected), split from `painter_bridge` for
/// the HR-18 file-LOC cap.
pub mod painter_bridge_shape_preview;
/// On-canvas wetness sheen veil (Watercolor render-path), split from `painter_bridge_overlays` for the
/// HR-18 file-LOC cap.
pub mod painter_bridge_upload;
pub mod painter_bridge_wetness;
pub mod painter_gpu_flatten;
pub mod painter_gpu_preview;
pub mod painter_lock;
/// Display gates, producer-handoff half (upload-plan refusals + the CPU→GPU→CPU dance on real
/// hardware) — split from the pipeline tests for the HR-18 file-LOC cap.
#[cfg(test)]
mod painter_preview_handoff_tests;
/// Display gates, a metade que MEDE — o preço de cada produtor pelo mesmo traço. Irmão do de cima,
/// cortado dele pelo teto de LOC da shell e por ASSUNTO (o que se AFIRMA × o que se MEDE).
#[cfg(test)]
mod painter_preview_measure;
/// Ownership gates: the shell's preview buffer is INDEPENDENT of the tool's canvas, so a plain stroke
/// stays footprint-bound (the tool keeps sole ownership) — split from the pipeline tests (HR-18).
#[cfg(test)]
mod painter_preview_ownership_tests;
/// Display gates: the preview slot (what the sprite shader samples) is held byte-equal to the
/// tool's composite across a stroke's whole life — phase D of the impasto smoke.
#[cfg(test)]
mod painter_preview_pipeline_tests;
/// O que a tela mostra DEPOIS de um undo — o report de resquício do smoke de 2026-07-25.
#[cfg(test)]
mod painter_preview_undo_tests;
/// A ponte do carimbo de pigmento para o dispositivo (doc 33 §S3) — a metade do lado do shell.
pub mod painter_stamp_device;
pub mod substrate_smoke;
pub mod taper_smoke;
pub mod wetpaint_smoke;

#[cfg(test)]
mod family_tests;

/// **O que esta família declara à shell** (`ph2d_app_host::AppFamily`).
///
/// ⛔⛔ **Uma família registada DECLARA roteador.** A catraca
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` morreu na Fase C, e a única ausência aceite é a da
/// família cuja cena vive numa crate irmã — que **não** é o caso desta: os seis roteadores estão
/// aqui, com a `env` lida aqui.
///
/// ⚠️ **Os `max_level` são CONTADOS, cada um no seu ficheiro** (CLAUDE.md §5.0), e a contagem tem
/// duas espécies nesta família:
///
/// | roteador | forma | `max_level` |
/// |---|---|---:|
/// | `PH2D_IMPASTO_SMOKE` | `match` com dois braços (`=2` abre 4096²) | [`impasto_smoke::NIVEIS`] = 2 |
/// | os outros **cinco** | `var_os(..).is_some()` — **presença** | `NIVEIS` = 1 |
///
/// ⛔ **`PH2D_PAINT_PERF`, `PH2D_PREVIEW_DIAG` e `PH2D_PREVIEW_DUMP` não estão aqui, e a ausência é
/// a decisão:** eles são DIAGNÓSTICO, não cenas. *Um roteador declarado diz ao dono que ele tem uma
/// cena para ver*, e mandá-lo correr um despejo de buffers seria uma promessa falsa.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "painter",
    routers: &[
        ph2d_app_host::SmokeRouter {
            env: "PH2D_IMPASTO_SMOKE",
            max_level: impasto_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_WETPAINT_SMOKE",
            max_level: wetpaint_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MASK_SMOKE",
            max_level: mask_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_SUBSTRATE_SMOKE",
            max_level: substrate_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_TAPER_SMOKE",
            max_level: taper_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_LINE_SMOKE",
            max_level: line_smoke::NIVEIS,
        },
    ],
};
