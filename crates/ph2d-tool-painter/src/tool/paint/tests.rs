//! **A raiz do corte de `tests.rs`** — o roteador do módulo e as fixturas que TODO filho partilha.
//!
//! Este arquivo tinha 23 603 linhas e 503 testes num só corpo. Ele foi cortado por
//! **responsabilidade** (um módulo por assunto, nunca por número de linhas), e o que ficou aqui é
//! exatamente o que não pertence a um assunto só:
//!
//! - os módulos irmãos declarados por `#[path]` (que moram ao lado, em `paint/`, e cujo caminho
//!   relativo continua a ser o diretório deste arquivo);
//! - as quatro fixturas que praticamente todo teste do módulo usa (`cp`, `white_canvas`, `frame`,
//!   `px`) e o `click` que várias famílias chamam — ⚠️ elas ficam AQUI de propósito: os módulos
//!   irmãos acima também as alcançam por `use super::*`, e duplicá-las seria criar duas respostas
//!   para a mesma pergunta;
//! - a lista dos filhos. O que cada um é está no `//!` dele, não nesta lista.

#[path = "impasto_pool_tests.rs"]
mod impasto_pool_tests;
#[path = "journal_delta_tests.rs"]
mod journal_delta_tests; // o delta do journal É o de dois snapshots (doc 28 §5.58.2, degrau 2 do S3)
#[path = "journal_tests.rs"]
mod journal_tests; // o journal descreve a TELA, e só ela (doc 28 §5.23, os 3 mecanismos do degrau 2)
#[path = "line_probe.rs"]
mod line_probe; // W0 do plano 38: o que um Solid pularia, o que ele acrescenta, e o preço da borda
#[path = "measure_boolean_cost.rs"]
mod measure_boolean_cost; // ...e o que a Operation cobra por cima dela
#[path = "measure_commit_cost.rs"]
mod measure_commit_cost; // …e de que é feito o CUSTO: fork, pen-up, commit, Ctrl+Z (doc 28 §5.13-§5.20)
#[path = "measure_dirty_overclaim.rs"]
mod measure_dirty_overclaim;
#[path = "measure_gpu_frontier.rs"]
mod measure_gpu_frontier; // o pool dos cinco planos do traço escreve o que a alocação escrevia
#[path = "ribbon_probe.rs"]
mod ribbon_probe; // W6 do plano 38: o orcamento da FITA — move, tique e o pen-up
#[path = "solid_deposit_tests.rs"]
mod solid_deposit_tests;
#[path = "solid_transaction_tests.rs"]
mod solid_transaction_tests; // …e o que a TRANSACAO garante: nada apagado, nada fora do retangulo
#[path = "spray_defaults_tests.rs"]
mod spray_defaults_tests; // W5: a primeira nuvem parece uma nuvem
#[path = "spray_probe.rs"]
mod spray_probe; // W5 do plano 38: o custo por evento de `n` marcas — o teto do Count
#[path = "thread_deposit_tests.rs"]
mod thread_deposit_tests;
#[path = "thread_probe.rs"]
mod thread_probe; // W3/W4 do plano 38: o custo por evento que os tetos do Sketchy e do Wire EXIGEM

#[path = "measure_penup_cost.rs"]
mod measure_penup_cost; // o que custa FECHAR um traço — a irmã do `measure_pendown_cost`

#[path = "look_watercolor_arc.rs"]
mod look_watercolor_arc; // o arco palido na concavidade: o oraculo e o RENDER, nao um escalar
#[path = "measure_impasto_cost.rs"]
mod measure_impasto_cost; // o que o CORPO da tinta custa (plano 26 §9); irmão do input_cost
#[path = "measure_input_cost.rs"]
mod measure_input_cost;
#[path = "measure_journal_cost.rs"]
mod measure_journal_cost; // captura por REGIÃO x fork do PLANO — o número que decide o S3 (doc 28 §7)
#[path = "measure_pendown_cost.rs"]
mod measure_pendown_cost; // o que COMEÇAR um traço custa (doc 28 §4.5); irmão do impasto_cost
#[path = "measure_rail_chips.rs"]
mod measure_rail_chips; // o que cada chip do rail FAZ no meio Digital — a medicao que decide o pill
#[path = "measure_relief_systems.rs"]
mod measure_relief_systems; // AUDITORIA: o custo dos DOIS sistemas de relevo (Enio 2026-08-10)
#[path = "measure_route_cost.rs"]
mod measure_route_cost; // quem RODA o deposito, e quanto essa escolha custa (doc 28)
#[path = "measure_shape_cost.rs"]
mod measure_shape_cost; // o que um MOVE de SHAPE EDITOR custa — o re-stamp da figura inteira
#[path = "measure_shape_system.rs"]
mod measure_shape_system;
#[path = "measure_solid_cost.rs"]
mod measure_solid_cost;
#[path = "measure_solid_shape.rs"]
mod measure_solid_shape; // …e o que ela DESENHA — a teia, o retangulo, os seis tipos // AUDITORIA: o que o Solid cobra sob Symmetry Circular + Tiling (Enio 2026-08-15) // …e quanto disso NÃO é o depósito — a máquina de shape sozinha
#[path = "measure_stroke_extent.rs"]
mod measure_stroke_extent; // o pen-up é função do que o traço COBRE — a fixture que faltava (§5.65)
#[path = "measure_stroke_owners.rs"]
mod measure_stroke_owners; // QUEM segura os planos quando um traço começa (doc 28 §7, a porta única)
#[path = "measure_undo_cost.rs"]
mod measure_undo_cost; // o que VOLTAR na história custa, e em qual metade (doc 28 §5.62)
#[path = "measure_watercolor_cost.rs"]
mod measure_watercolor_cost; // de que é feito um MOVE de aquarela (doc 28 §7); irmão do impasto_cost
#[path = "measure_watercolor_pour.rs"]
mod measure_watercolor_pour; // o que o pour cobra e o que a rota do quadro muda (doc 28 §5.72)
#[path = "measure_watercolor_stamp.rs"]
mod measure_watercolor_stamp;
#[path = "measure_watercolor_water_edge.rs"]
mod measure_watercolor_water_edge; // a borda da AGUA carregada recebe o mesmo AA que a do pigmento? // o carimbo é função da PEGADA ou do estado do CANVAS? (doc 32 §5)
#[path = "measure_wetpaint_cost.rs"]
mod measure_wetpaint_cost; // de que é feito um MOVE de Wet Paint (doc 28 §7, frente V)
#[path = "measure_window_premise.rs"]
mod measure_window_premise;
#[path = "selection_trace_tests.rs"]
mod selection_trace_tests; // a varredura por FAIXAS traca o que o flood pixel-a-pixel tracava
#[path = "shape_draft_tests.rs"]
mod shape_draft_tests; // o meio caro renderiza em REPOUSO — a lei do rascunho sob a mao
#[path = "stamp_banded_tests.rs"]
mod stamp_banded_tests; // o lote em bandas pinta o que o laco serial pintava (doc 28 §5.78)
#[path = "stamp_banded_work_tests.rs"]
mod stamp_banded_work_tests; // ...e as bandas sao cortadas por TRABALHO - o lote ESPARSO
#[path = "stroke_boolean_tests.rs"]
mod stroke_boolean_tests; // o composite booleano roda na janela das FORMAS, nao na do canvas
#[path = "stroke_outline_tests.rs"]
mod stroke_outline_tests; // o CONTORNO e o que se ve E o que se clica
#[path = "undo_confine_tests.rs"]
mod undo_confine_tests; // um Ctrl+Z repinta so' o que ele mudou (doc 28 §5.63)
#[path = "undo_live_base_tests.rs"]
mod undo_live_base_tests;
#[path = "watercolor_smudge_gate_tests.rs"]
mod watercolor_smudge_gate_tests; // o smudge não forka o canvas (doc 28 §5.73) // a premissa do S3, MEDIDA: o vivo serve de base p/ o delta? (doc 28 §5.20)

mod brush_panel;
mod curve_editor;
mod deform;
mod edit_modes;
mod ellipse_and_polygon;
mod grain_and_stencil;
mod impasto_body;
mod line_editor;
mod parked_shapes;
mod protection_and_layer_mask;
mod selection_editing;
mod shape_per_layer_color;
mod shape_silhouette;
mod stroke_and_session;
mod texture_and_tiling;
mod watercolor_look;
mod watercolor_parity;
mod watercolor_seams;
mod watercolor_session;
mod watercolor_water;

use super::*;
use crate::tool::paint::{ImpastoLight, MAX_IMPASTO_LIGHTS};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::{DepthSource, DrawTo, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A `PainterTool` sourced with a white opaque `size`×`size` canvas (one
/// active raster layer) and a small hard black brush for crisp assertions.
fn white_canvas(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: radius,
        hardness: 1.0, // hard disk → deterministic centre
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        // These tests assert FULL-coverage pixels to verify painting mechanics
        // (alpha-lock / undo / blend). The Blender-default "Adjust Strength for
        // Spacing" attenuates a lone dab below full opacity, so opt out here — the
        // attenuation behaviour has its own dedicated engine test.
        space_attenuation: false,
        ..Default::default()
    };
    // Seed every per-mode slot with this hard-disk fixture brush so a mode switch (e.g. into Mask) keeps
    // it instead of loading that tool's independent default (the "Sync with other tools" model). Tests
    // that exercise the independent/linked behaviour itself set their slots explicitly.
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

/// **Close a FRAME.** The app delivers the frame's pointer events and *then* ticks the active tool
/// (`render_loop` ~698, then ~1198, both before the preview upload at ~3397). Since 2026-08-02 the
/// watercolor's optical reconstruction is **owed to that tick** rather than run inside every Move
/// (the tick composites when the frame was not `parked`), because the wash window is padded by the
/// influence radius and a
/// per-event reconstruction made the same drawing cost up to 2,56× more on a high-Hz device.
///
/// ⚠️ So a watercolor fixture that drives Moves and never lands here is measuring a wash that **never
/// recomposites** — the assertion it makes about live pixels is about the old cadence, not about the
/// product. Every gate below that reads canvas pixels mid-stroke closes its frames through here.
fn frame(t: &mut PainterTool) {
    t.paint_tick(1.0 / 60.0);
}

fn px(t: &PainterTool, size: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * size + x) * 4) as usize;
    [
        t.canvas_rgba[i],
        t.canvas_rgba[i + 1],
        t.canvas_rgba[i + 2],
        t.canvas_rgba[i + 3],
    ]
}

fn click(t: &mut PainterTool, x: f32, y: f32) {
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
}
