//! ⭐⭐⭐ **AS SECÇÕES DO PAINEL DE FÍSICA COMO UMA LISTA — a ordem do artista e o tema de cada uma.**
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas no título e a pega de dez pontos nascerem
//! no Inspector e chegarem ao Vector e ao Painter: *«siga com os outros painéis»*. O corpo passa a
//! ser um [`PlanoCtx`] — a lei da ordem, do tema por secção, do corredor entre cartões, da marca de
//! queda e do fantasma é a partilhada; aqui só mora QUEM é o quê.
//!
//! ⭐ **As nove secções se ARRASTAM, e nenhuma é fixa.** A lei das fixas (no Painter: a Máscara e o
//! meio da tinta) é *«ela reinterpreta o que está abaixo dela»*, e aqui nenhuma o faz — cada uma
//! descreve uma faceta independente do mundo (gravidade, solver, ar, amortecimento, sono) ou do
//! ponteiro (Interaction, Joints), mais a matriz de camadas e o Debug. Nenhuma está DENTRO de
//! outra. ⚠️ O Debug carrega o *Reset to Defaults* no fundo dele — ver
//! [`super::body::debug_section`] para porque ele também se arrasta.
//!
//! ⚠️ **A lista é a MESMA de sempre** ([`rows::SECTIONS`] seguida de [`rows::HAND_PAINTED_SECTIONS`],
//! a [`rows::section_header_ids`] que o `populate` regista e o `event` despacha) — a ordem NATURAL
//! dela é a de antes desta wave, e é a que o artista vê enquanto não arrastar nada.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan_ctx::PlanoCtx;
use ph2d_tokens::Theme;

use crate::rows;
use crate::state::PhysicsSnapshot;

/// Paint every section, in the artist's order, each in its own theme. Returns the `y` it ended at.
pub(super) fn paint_sections(
    ctx: &mut PaintCtx,
    painel: Theme,
    snapshot: &PhysicsSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let snap = *snapshot;
    let mut plano = PlanoCtx::new();
    for section in rows::SECTIONS {
        plano.seccao(section.id, move |ctx, tema, y| {
            super::body::table_section(ctx, tema, section, &snap, (x, w, y))
        });
    }
    // ⚠️ As quatro pintadas à mão, pela ordem de [`rows::HAND_PAINTED_SECTIONS`] — o `debug_assert`
    //    abaixo prende as duas listas uma à outra, e o gate de costura
    //    `toda_seccao_pintada_entra_no_livro_com_pega` (tests/it) mede a mesma coisa pelo lado do
    //    livro do quadro: uma secção pintada fora desta tabela não teria pega registada.
    type Pintor = fn(&mut PaintCtx, Theme, &PhysicsSnapshot, (f32, f32, f32)) -> f32;
    let a_mao: [(ph2d_a11y::NodeId, Pintor); 4] = [
        (
            crate::ids::PHYSICS_SEC_INTERACT,
            super::body::interact_section,
        ),
        (crate::ids::PHYSICS_SEC_JOINT, super::body::joint_section),
        (crate::ids::PHYSICS_SEC_LAYERS, super::body::layers_section),
        (crate::ids::PHYSICS_SEC_DEBUG, super::body::debug_section),
    ];
    debug_assert!(
        a_mao
            .iter()
            .map(|(id, _)| *id)
            .eq(rows::HAND_PAINTED_SECTIONS.iter().copied()),
        "as secções pintadas à mão divergiram da tabela que o populate e o event leem"
    );
    for (id, pinta) in a_mao {
        plano.seccao(id, move |ctx, tema, y| pinta(ctx, tema, &snap, (x, w, y)));
    }
    // ⚠️ O cartão da última secção fecha-se DENTRO do `corre` (`Corredor::fecha_a_ultima`) — um
    //    remendo local aqui fechá-lo-ia duas vezes.
    plano.corre(
        ctx,
        ph2d_editor_core::ids::PHYSICS_PANEL,
        painel,
        x,
        w,
        ph2d_editor_core::panel::rows::section_header_h(),
        y_in,
    )
}
