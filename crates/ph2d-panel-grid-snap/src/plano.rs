//! ⭐⭐⭐ **AS SECÇÕES DO PAINEL DA GRELHA COMO UMA LISTA — a ordem do artista e o tema de cada uma.**
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas no título e a pega de dez pontos nascerem
//! no Inspector e chegarem ao Vector e ao Painter: *«siga com os outros painéis»*. O corpo passa a
//! ser um [`PlanoCtx`] — a lei da ordem, do tema por secção, do corredor entre cartões, da marca de
//! queda e do fantasma é a partilhada; aqui só mora QUEM é o quê.
//!
//! | lugar | quem |
//! |---|---|
//! | **bloco** (sem título, fica no topo) | o botão grande *Snap* |
//! | **móvel** (arrasta-se e muda de tema) | *Grid Kind* · *Target* · *Display* · *Inspect* |
//!
//! ⭐ **Nenhuma é fixa, e o porquê é a lei das fixas:** uma secção fica no lugar quando
//! REINTERPRETA o que está abaixo dela (no Painter, a Máscara e o meio da tinta). O *Grid Kind*
//! governa só a configuração por-tipo, que mora DENTRO dele; o *Target*, o *Display* e o *Inspect*
//! pintam os mesmos controlos seja qual for o tipo. Nenhuma está DENTRO de outra — o corpo do
//! *Inspect* (`grid_snap::inspect::paint_body`) não desenha cabeçalho nenhum.
//!
//! ⚠️ **O *Snap* é um BLOCO e não uma secção** — não tem título, logo não tem onde o botão direito
//! abrir menu nem onde a pega morar; e o plano pinta os blocos PRIMEIRO, que é onde ele sempre
//! esteve (o interruptor mestre do painel no topo).

use ph2d_a11y::NodeId;
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan_ctx::PlanoCtx;
use ph2d_tokens::Theme;

/// ⭐ **As secções do painel que se ARRASTAM**, pela ordem natural — a MESMA lista que o
/// `populate` usa para registar as pegas.
pub(crate) const SECCOES: [NodeId; 4] = [
    crate::ids::GS_SEC_KIND,
    crate::ids::GS_SEC_TARGET,
    crate::ids::GS_SEC_DISPLAY,
    ph2d_editor_core::grid_snap::ids::GS_INSPECT_HEADER,
];

/// Uma secção do corpo: pinta-a no tema que recebe e devolve o `y` seguinte.
type Pintor = fn(&mut PaintCtx<'_>, Theme, &GridSnapState, f32, f32, f32) -> f32;

/// Paint the body — the Snap block, then the four sections in the artist's order, each in its own
/// theme. Returns the `y` it ended at.
pub(crate) fn paint_sections(
    ctx: &mut PaintCtx<'_>,
    painel: Theme,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    top: f32,
) -> f32 {
    let mut plano = PlanoCtx::new();

    // ─── Snap (BIG individual toggle) — o bloco do topo, no tema do PAINEL ───
    plano.bloco(move |ctx, theme, y| {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        crate::paint_helpers::paint_snap_top_toggle(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        )
    });

    let pintores: [Pintor; 4] = [
        crate::paint_body_sections::paint_grid_kind_section,
        crate::paint_body_sections::paint_target_section,
        crate::paint_body_sections::paint_display_section,
        crate::paint_body_sections::paint_inspect_section,
    ];
    for (id, pinta) in SECCOES.into_iter().zip(pintores) {
        plano.seccao(id, move |ctx, tema, y| {
            pinta(ctx, tema, state, inner_x, inner_w, y)
        });
    }

    // ⚠️ O cartão da última secção fecha-se DENTRO do `corre` (`Corredor::fecha_a_ultima`).
    plano.corre(
        ctx,
        painel,
        inner_x,
        inner_w,
        ph2d_editor_core::panel::rows::section_header_h(),
        top,
    )
}
