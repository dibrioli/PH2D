//! ⭐⭐⭐ **AS SECÇÕES DO TUNING COMO UMA LISTA — todas se arrastam e todas mudam de tema.**
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas no título e a pega de dez pontos nascerem
//! no Inspector e chegarem ao Vector e ao Painter: *«siga com os outros painéis»*. O corpo passa a
//! ser um [`PlanoCtx`](ph2d_editor_core::panel::section_plan_ctx::PlanoCtx) — a lei da ordem, do
//! tema, da marca de queda e do fantasma é a partilhada; aqui só mora QUEM é o quê.
//!
//! ⭐ **As seis são MÓVEIS, e nenhuma é fixa nem aninhada** — os cinco grupos de botões do motor
//! (Paint · Water · Physics · Tools · Paper) e o cartão *Experimental* são assuntos paralelos: nenhum
//! reinterpreta o que está abaixo dele (a razão que prende a Máscara do Painter), e nenhum vive
//! dentro de outro. ⚠️ O *olho* do Paper e o repor de cada grupo moram NO cabeçalho e viajam com
//! ele — encostados à esquerda da pega.
//!
//! ⚠️ **O painel tem a SUA dobra** (o `event` despacha o clique do título como `toggle_collapsed`),
//! e ela continua a valer: o livro do quadro regista o título sob o MESMO id da secção, que é o que
//! o despacho do botão esquerdo lê.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan;
use ph2d_editor_core::widget::SectionHeader;
use ph2d_editor_core::zones::Rect;

use crate::rows;

/// O id do cartão *Experimental* — o sexto cabeçalho, que não é um grupo de botões do motor.
pub(crate) const EXPERIMENTAL: NodeId = ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[5];

/// ⭐ **As secções que se ARRASTAM**, pela ordem natural em que o corpo as pinta — os cinco grupos
/// da tabela e o *Experimental*. ⚠️ Derivada da [`rows::SECTIONS`]: um grupo novo entra aqui pelo
/// mesmo commit que o faz existir.
pub(crate) fn seccoes_moveis() -> impl Iterator<Item = NodeId> {
    rows::SECTIONS
        .iter()
        .map(|s| s.header)
        .chain(std::iter::once(EXPERIMENTAL))
}

/// ⭐ **O cabeçalho da secção `id`** — a dobra viva de sempre e a PEGA (todas se arrastam).
#[must_use]
pub(crate) fn cabecalho(ctx: &PaintCtx, id: NodeId, label: &str) -> SectionHeader {
    section_plan::cabecalho(ctx.host.store(), id, label)
}

/// ⭐ **Regista o cabeçalho da secção `id` no livro do quadro** — o título (que o clique esquerdo
/// dobra e o direito abre no menu de tema) e a pega.
pub(crate) fn regista(ctx: &mut PaintCtx, id: NodeId, head: Rect) {
    section_plan::regista_cabecalho(ctx.host.hit_index_mut(), id, head);
}

/// A largura que a ponta direita de um cabeçalho guarda para a pega — quem pinta um controlo ali
/// (o repor do grupo, o olho do Paper) começa à esquerda dela.
#[must_use]
pub(crate) fn largura_da_pega() -> f32 {
    ph2d_editor_core::widget::section_grip::grip_slot_w_px()
}
