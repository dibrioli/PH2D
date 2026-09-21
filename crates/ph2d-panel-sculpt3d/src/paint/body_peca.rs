//! ⭐⭐ **O QUE SE FAZ COM A PEÇA** — a lista de peças da cena, os verbos que a mexem, e o gesto
//! que a manda para dentro de um sprite.
//!
//! Irmão (`#[path]`) do [`super::body`], e o corte é por RESPONSABILIDADE e não por tamanho: lá
//! mora *como o barro é moldado e mostrado* (a ferramenta, o pincel, o espelho, a topologia, o
//! sombreamento); aqui *o que se faz com a peça inteira* — acrescentar, duplicar, apagar, isolar e
//! ASSAR. ⚠️ **As duas metades crescem por motivos diferentes**, que é a mesma frase com que o
//! `body.rs` se separou do `paint.rs`.
//!
//! ⚠️ O gatilho foi o tecto de LOC (600 numa crate de painel), e ele mordeu no dia em que a fileira
//! da LENTE entrou na secção do sombreamento — ⛔ a cura é **CORTE**, nunca uma entrada nova no
//! `FILE_OVERAGE_OK` (CLAUDE.md §5.0).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use super::widgets::{
    self, command, command_na_celula, header, labelled_seg, readout, row_of_two, toggle,
    toggle_na_celula,
};

use crate::state::Sculpt3dSnapshot;

/// Os rótulos das quatro primitivas, na ordem dos comandos `Add*`.
const ADD_LABELS: [&str; 4] = [
    "panel.sculpt3d.add.sphere",
    "panel.sculpt3d.add.cube",
    "panel.sculpt3d.add.cylinder",
    "panel.sculpt3d.add.torus",
];

/// **A CENA** — a lista de peças e os verbos que a mexem.
pub(super) fn paint_scene(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let (fold, mut y) = header(
        ctx,
        crate::ids::SCULPT3D_SEC_SCENE,
        tr("panel.sculpt3d.section.scene"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let add: Vec<&str> = ADD_LABELS.iter().map(|k| tr(k)).collect();
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.add"),
        &crate::ids::SCULPT3D_ADD,
        &add,
        usize::MAX, // gestos, não um modo
        x,
        w,
        y,
    );
    y = row_of_two(
        ctx,
        (
            crate::ids::SCULPT3D_DUPLICATE,
            tr("panel.sculpt3d.duplicate"),
        ),
        (crate::ids::SCULPT3D_DELETE, tr("panel.sculpt3d.delete")),
        x,
        w,
        y,
    ) + gap;
    // O Isolate é o único desta fileira com ESTADO — ele fica aceso enquanto a
    // cena está reduzida a uma peça, senão o artista perde quatro objetos e não
    // tem na tela nada que explique por quê.
    let half = (w - gap) * 0.5;
    toggle_na_celula(
        ctx,
        crate::ids::SCULPT3D_ISOLATE,
        tr("panel.sculpt3d.isolate"),
        snap.isolated,
        x,
        half,
        y,
    );
    y = command_na_celula(
        ctx,
        crate::ids::SCULPT3D_MERGE,
        tr("panel.sculpt3d.merge"),
        x + half + gap,
        half,
        y,
    ) + gap;
    y = readout(
        ctx,
        &format!(
            "{}: {}   {}: {}",
            tr("panel.sculpt3d.pieces"),
            snap.pieces,
            tr("panel.sculpt3d.verts"),
            snap.verts
        ),
        x,
        w,
        y,
    );
    widgets::end_fold(ctx, fold, y + gap)
}

/// **A ENTREGA** — a forma escrita num objeto da cena 2D (`docs/3D/02.2`, o
/// objetivo 2 do módulo).
///
/// ⚠️ **Seção própria, e por último.** As cinco de cima descrevem *como a
/// escultura é*; esta descreve *o que sai dela*, e é o gesto mais raro do painel
/// — que é exatamente a lei de ordenação que o doc do topo declara. Uma linha na
/// cauda do sombreamento a colaria no *Bake Occlusion*, e os dois carregam a
/// palavra **bake** significando coisas diferentes: aquele mede um canal e o
/// escreve na MALHA, este escreve a forma inteira num SPRITE.
///
/// ⚠️ **O botão é SEMPRE pintado, e a dica é que some.** Esconder o botão sem
/// alvo tornaria a única entrega do módulo invisível justamente para quem ainda
/// não sabe que ela existe — que é a queixa que ele veio resolver (até aqui o
/// gesto tinha uma porta só, o `Shift+B`, e nada na tela a mencionava). A
/// condição é DITA, no molde do `ao_stale`: a linha só existe quando há o que
/// avisar, porque um aviso permanente vira moldura.
pub(super) fn paint_bake(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (fold, mut y) = header(
        ctx,
        crate::ids::SCULPT3D_SEC_BAKE,
        tr("panel.sculpt3d.section.bake"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    y = command(
        ctx,
        crate::ids::SCULPT3D_BAKE_SPRITE,
        tr("panel.sculpt3d.bake_sprite"),
        x,
        w,
        y,
    );
    if !snap.has_bake_target {
        y = readout(ctx, tr("panel.sculpt3d.bake_sprite.hint"), x, w, y);
    }
    // ⭐⭐ **A LEI do objecto assado — e ela só é pintada quando há objecto assado.**
    // Sem canais não há lei para escolher, e um selector que não governa nada é o
    // controlo morto que esta crate já pagou sete vezes. ⚠️ Os rótulos vêm do
    // RETRATO e não daqui: quem define as leis é outra crate, e dois nomes
    // escritos no painel seriam a segunda ortografia da mesma lei.
    if let Some(escolhida) = snap.lei_do_alvo {
        let labels: Vec<&str> = snap.lei_rotulos.iter().map(|k| tr(k)).collect();
        y = widgets::labelled_seg(
            ctx,
            tr("panel.sculpt3d.bake_law"),
            crate::ids::SCULPT3D_SEC_BAKE,
            &crate::ids::SCULPT3D_BAKE_LAW,
            &labels,
            escolhida,
            x,
            w,
            y,
        );
    }
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}
