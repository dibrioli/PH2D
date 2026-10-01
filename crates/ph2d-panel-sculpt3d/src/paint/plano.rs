//! ⭐⭐⭐ **AS SECÇÕES DO PAINEL DA ESCULTURA COMO UMA LISTA — quais se arrastam, qual fica, e o
//! TEMA em que cada uma se pinta.**
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas no título e a pega de dez pontos nascerem
//! no Inspector e chegarem ao Vector e ao Painter: *«siga com os outros painéis»*. O corpo passa a
//! ser um [`PlanoCtx`](ph2d_editor_core::panel::section_plan_ctx::PlanoCtx) — a lei da ordem, do
//! tema, da marca de queda e do fantasma é a partilhada; aqui só mora QUEM é o quê.
//!
//! | lugar | o que faz | quem |
//! |---|---|---|
//! | FIXA | muda de tema e fica no lugar | a `Tool` |
//! | MÓVEL | arrasta-se e muda de tema | `Brush` · `Symmetry` · `Topology` · `Shading` · `Scene` · `Bake` |
//!
//! ⛔ **A `Tool` fica por uma razão de PRODUTO, não de conforto:** é ela que escolhe o pincel em
//! mãos, e o pincel em mãos decide que fileiras existem em todas as outras (as do `Brush`, as
//! próprias de cada verbo, a pista do pente na `Topology`). Deixá-la descer seria pôr o governante
//! debaixo dos governados — a mesma razão que prende a Máscara do Painter.
//!
//! ⚠️ **Não há secção ANINHADA neste painel** — as sete são de topo, e todas pintam o cabeçalho pela
//! MESMA porta ([`super::widgets::header`]).
//!
//! # ⭐⭐ O tema em que uma secção se pinta é um ESCOPO, não um argumento
//!
//! Os widgets deste painel pediam o tema ao host (`ctx.host.theme()`) — e o host responde o tema do
//! PAINEL. Uma secção a que o artista deu outro tema tem de pintar os controlos NELE (o `retheme` do
//! plano só recolore o CARTÃO), senão um texto escuro do tema do painel pousa num cartão escuro do
//! tema da secção. ⚠️ **Enfiar o tema na assinatura** custaria ~100 sítios de chamada e ~30
//! assinaturas, para responder cem vezes a uma pergunta que o plano responde UMA por secção — a
//! mesma conta que pôs o livro dos cartões num `thread_local` em vez de na assinatura de cada
//! pintor ([`ph2d_editor_core::widget::section_cards`]). ⇒ [`no_tema`] abre o escopo durante a
//! tarefa da secção e [`tema`] é a ÚNICA porta por onde um pintor deste painel pergunta o tema.
//! ⛔ Um `ctx.host.theme()` num pintor de secção é um controlo pintado no tema errado, e há censo
//! a recusá-lo (`plano_tests.rs`).

use std::cell::Cell;

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan;
use ph2d_editor_core::widget::SectionHeader;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

/// ⭐ **A secção que fica no LUGAR e muda de tema** — a `Tool`. Ver o doc do módulo.
pub(crate) const SECCAO_FIXA: NodeId = crate::ids::SCULPT3D_SEC_TOOL;

/// ⭐ **As secções que se ARRASTAM**, pela ordem natural em que o corpo as pinta.
pub(crate) const SECCOES_MOVEIS: [NodeId; 6] = [
    crate::ids::SCULPT3D_SEC_BRUSH,
    crate::ids::SCULPT3D_SEC_SYMMETRY,
    crate::ids::SCULPT3D_SEC_TOPOLOGY,
    crate::ids::SCULPT3D_SEC_SHADING,
    crate::ids::SCULPT3D_SEC_SCENE,
    crate::ids::SCULPT3D_SEC_BAKE,
];

/// ⭐ **O cabeçalho da secção `id`, pelo lugar dela** — com a pega se ela se arrasta.
#[must_use]
pub(super) fn cabecalho(ctx: &PaintCtx, id: NodeId, label: &str) -> SectionHeader {
    let store = ctx.host.store();
    if SECCOES_MOVEIS.contains(&id) {
        section_plan::cabecalho(store, id, label)
    } else {
        section_plan::cabecalho_fixo(store, id, label)
    }
}

/// ⭐ **Regista o cabeçalho da secção `id` no livro do quadro, pelo lugar dela** — o título (que o
/// clique esquerdo dobra — a dobra é deste painel, e o `event` lê-a pelo MESMO id — e o direito
/// abre no menu de tema) e, se ela se arrasta, a pega.
pub(super) fn regista(ctx: &mut PaintCtx, id: NodeId, head: Rect) {
    let hit = ctx.host.hit_index_mut();
    if SECCOES_MOVEIS.contains(&id) {
        section_plan::regista_cabecalho(hit, id, head);
    } else {
        section_plan::regista_cabecalho_fixo(hit, id, head);
    }
}

thread_local! {
    /// O tema da secção que se pinta AGORA — `None` fora de uma ([`no_tema`]).
    static TEMA: Cell<Option<Theme>> = const { Cell::new(None) };
}

/// ⭐⭐ **O tema em que um pintor deste painel pinta** — o da secção em curso, ou o do painel fora
/// de uma. A ÚNICA porta: ver o doc do módulo.
#[must_use]
pub(crate) fn tema(ctx: &PaintCtx) -> Theme {
    TEMA.with(Cell::get).unwrap_or_else(|| ctx.host.theme())
}

/// Repõe o escopo anterior ao sair — também num pânico, senão o tema de uma secção vazaria para o
/// quadro seguinte.
struct Escopo(Option<Theme>);

impl Drop for Escopo {
    fn drop(&mut self) {
        TEMA.with(|t| t.set(self.0));
    }
}

/// ⭐⭐ **A tarefa do plano para uma secção** — pinta-a com o escopo do TEMA dela aberto.
///
/// ⚠️ O cartão da ÚLTIMA secção fecha-o o próprio plano (`Corredor::fecha_a_ultima`): o
/// `end_section_cards` só pinta os cartões que já fecharam.
pub(super) fn no_tema<'s>(
    pinta: impl for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, f32) -> f32 + 's,
) -> impl for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, Theme, f32) -> f32 + 's {
    move |ctx: &mut PaintCtx<'_>, tema: Theme, y: f32| {
        let _escopo = Escopo(TEMA.with(|t| t.replace(Some(tema))));
        pinta(ctx, y)
    }
}

#[cfg(test)]
#[path = "plano_tests.rs"]
mod tests;
