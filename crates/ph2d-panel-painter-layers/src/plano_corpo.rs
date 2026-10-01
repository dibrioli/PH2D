//! ⭐⭐⭐ **AS SECÇÕES DO CORPO DO PINCEL — quais se arrastam, quais ficam, e o cabeçalho de cada.**
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas no título e a pega de dez pontos nascerem
//! no Inspector e chegarem ao Vector: *«siga com os outros painéis»*. O corpo do pincel passa a ser
//! um [`PlanoCtx`](ph2d_editor_core::panel::section_plan_ctx::PlanoCtx) — a lei da ordem, do tema,
//! da marca de queda e do fantasma é a partilhada; aqui só mora QUEM é o quê.
//!
//! ⚠️ **Três lugares, e a resposta é deste painel e de mais ninguém:**
//!
//! | lugar | o que faz | quem |
//! |---|---|---|
//! | [`Lugar::Movel`] | arrasta-se e muda de tema | as oito da aparência (Randomize … Tiling) |
//! | [`Lugar::Fixa`] | muda de tema e fica no lugar | a Máscara e a secção do MEIO da tinta |
//! | [`Lugar::Dentro`] | nenhum dos dois | uma secção DENTRO de outra (a rampa do Grain) |
//!
//! ⛔ **As fixas ficam por uma razão de PRODUTO, não de conforto:** a Máscara e a secção do meio
//! reinterpretam tudo o que está abaixo delas (o chip do *Paint Mode* senta-se ACIMA do que governa
//! — a lei escrita em `paint_brush_sections.rs`). Deixá-las descer seria pôr o governante debaixo
//! dos governados.
//!
//! ⛔ **E uma secção ANINHADA não entra no livro do quadro:** se entrasse, o botão direito
//! abrir-lhe-ia um menu de tema que o plano não aplica, e a pega arrastá-la-ia para uma ordem que o
//! plano não lê — dois controlos mortos com cara de feature.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan;
use ph2d_editor_core::widget::SectionHeader;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::ids as pids;

/// ⭐ **As secções do corpo que se ARRASTAM**, pela ordem natural em que o corpo as pinta.
pub(crate) const SECCOES_MOVEIS: [NodeId; 8] = [
    pids::PAINTER_BRUSH_RANDOMIZE_SECTION,
    pids::PAINTER_SHAPE_SECTION,
    pids::PAINTER_SHAPE_RAMP_SECTION,
    pids::PAINTER_WATERCOLOR_PAPER_SECTION,
    pids::PAINTER_BRUSH_TEXTURE_SECTION,
    pids::PAINTER_BRUSH_STROKE_SECTION,
    pids::PAINTER_BRUSH_SYMMETRY_SECTION,
    pids::PAINTER_BRUSH_TILING_SECTION,
];

/// ⭐ **As secções do corpo que ficam no LUGAR e mudam de tema.**
pub(crate) const SECCOES_FIXAS: [NodeId; 4] = [
    pids::PAINTER_MASK_SECTION,
    pids::PAINTER_WATERCOLOR_SECTION,
    pids::PAINTER_IMPASTO_SECTION,
    pids::PAINTER_WETPAINT_SECTION,
];

/// O lugar de uma secção no corpo do pincel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Lugar {
    /// Arrasta-se e muda de tema.
    Movel,
    /// Muda de tema e fica no lugar.
    Fixa,
    /// Dentro de outra — nem uma coisa nem outra.
    Dentro,
}

/// O lugar da secção `id`.
#[must_use]
pub(crate) fn lugar(id: NodeId) -> Lugar {
    if SECCOES_MOVEIS.contains(&id) {
        Lugar::Movel
    } else if SECCOES_FIXAS.contains(&id) {
        Lugar::Fixa
    } else {
        Lugar::Dentro
    }
}

/// ⭐ **O cabeçalho da secção `id`, pelo lugar dela** — com a pega se se arrasta.
#[must_use]
pub(crate) fn cabecalho(ctx: &PaintCtx, id: NodeId, label: &str) -> SectionHeader {
    let store = ctx.host.store();
    match lugar(id) {
        Lugar::Movel => section_plan::cabecalho(store, id, label),
        Lugar::Fixa => section_plan::cabecalho_fixo(store, id, label),
        Lugar::Dentro => SectionHeader::new(id, label)
            .collapsible(!store.is_collapsed(id))
            .open_t(store.section_open_live(id)),
    }
}

/// ⭐ **Regista o cabeçalho da secção `id`, pelo lugar dela** — no livro do quadro se é do corpo;
/// só como alvo de dobra se está dentro de outra.
pub(crate) fn regista(ctx: &mut PaintCtx, id: NodeId, head: Rect) {
    let hit = ctx.host.hit_index_mut();
    match lugar(id) {
        Lugar::Movel => section_plan::regista_cabecalho(hit, id, head),
        Lugar::Fixa => section_plan::regista_cabecalho_fixo(hit, id, head),
        Lugar::Dentro => hit.register(id, head),
    }
}

/// A largura que a ponta direita do cabeçalho `id` guarda para a pega — `0` se ele não a tem.
#[must_use]
pub(crate) fn largura_da_pega(id: NodeId) -> f32 {
    if lugar(id) == Lugar::Movel {
        ph2d_editor_core::widget::section_grip::grip_slot_w_px()
    } else {
        ph2d_tokens::Spacing::Md.px()
    }
}
