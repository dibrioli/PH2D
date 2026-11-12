//! **Os cartões abertos do Platform Player (Walk, Jump)** e as duas linhas de editor do Jump: a caixa
//! «Show Jump Arc» e o seletor «Feel» (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`).
//! Irmão de `player_cartoes.rs` pelo tecto de LOC.

use super::*;
use crate::sections::rows::seg_row_misto;

/// Um cartão sempre aberto com as rows `lista`; o Jump leva também as linhas do arco e do Feel.
pub(super) fn cartao(p: &mut Pintor<'_>, y: f32, titulo: &'static str, lista: &[NodeId]) -> f32 {
    let pulo = lista == JUMP.as_slice();
    // Fora dos cartões a moldura mede-se pela CONTAGEM de rows: as duas do pulo contam.
    let n = lista.iter().filter(|id| p.shown(**id)).count() + if pulo { 2 } else { 0 };
    let (theme, x, w, store, info, coluna) = (p.theme, p.x, p.w, p.store, p.info, p.coluna);
    let hit = &mut *p.hit;
    card_frame_with(
        p.scene,
        p.text,
        theme,
        x,
        w,
        y,
        TextKey::new(titulo).tr(),
        n,
        |scene, text, ix, iw, ry| {
            let mut q = Pintor {
                scene,
                text,
                theme,
                hit,
                store,
                x: ix,
                w: iw,
                info,
                coluna,
            };
            let fim = q.rows(ix, iw, ry, lista);
            if pulo {
                linhas_do_pulo(&mut q, fim)
            } else {
                fim
            }
        },
    )
}

/// «Show Jump Arc» e «Feel». Com `feel = None` nenhum segmento acende: os números foram mexidos à mão.
fn linhas_do_pulo(p: &mut Pintor<'_>, y: f32) -> f32 {
    let y = caixa(
        p,
        y,
        ids::INSP_PLAYER_SHOW_ARC,
        TextKey::new("component.field.platform_player.62").tr(),
        p.info.show_jump_arc,
    );
    let rotulo = TextKey::new("panel.inspector.player.feel");
    let nomes = [
        tr("panel.inspector.player.floaty"),
        tr("panel.inspector.player.snappy"),
        tr("panel.inspector.player.heavy"),
    ];
    let sec = ph2d_editor_core::property_row::Seccao::medida(p.text, 1, &[rotulo.tr()]);
    seg_row_misto(
        p.scene,
        p.text,
        p.theme,
        p.hit,
        p.store,
        p.x,
        p.w,
        y,
        rotulo,
        (ids::INSP_PLAYER_FEEL, &ids::INSP_PLAYER_FEEL_IDS, &nomes),
        p.info.feel.map(usize::from),
        sec,
        ph2d_tokens::control_gap_px(),
    )
}
