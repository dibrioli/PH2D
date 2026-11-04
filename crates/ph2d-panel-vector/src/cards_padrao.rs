//! ⭐ **Os dois cartões Pattern** (fatia G5, `docs/CartoesUI/12_VECTOR_PLANO.md`): uma função, dois
//! sujeitos (`slot` 0 = o preenchimento, 1 = o traço), como no corpo de antes. Mesmos ids e eventos, e a
//! mesma lei: só a FASE (`Shift`) sai no `Clamp`, e o desfasamento só existe em Brick e Column.

use crate::cards_fx::{com_unidade, fila};
use crate::cards_tinta::{caixa, escolha, par_de_botoes, seg, semeia, spec};
use crate::ids::TexPatKnob as K;
use crate::paint_sections::texture_pattern::kid;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{TextKey, tr};

const PASSO_FINO: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento, o do `populate`
const SETA_DO_VAO: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento, o do `populate`
/// O `Mode` que não repete: uma cópia enquadrada, e a fase sem quem a leia.
const CLAMP: u8 = 2;

const TILES: [TextKey; 4] = [
    TextKey::new("panel.vector.texpat.grid"),
    TextKey::new("panel.vector.texpat.brick"),
    TextKey::new("panel.vector.texpat.column"),
    TextKey::new("panel.vector.texpat.hex"),
];
const MODES: [TextKey; 3] = [
    TextKey::new("panel.vector.texpat.tile"),
    TextKey::new("panel.vector.texpat.mirror"),
    TextKey::new("panel.vector.texpat.clamp"),
];

/// Com o store MUTÁVEL, antes de pintar: as marcas, e as sementes que o corpo antigo dava ao pintar.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for slot in 0..crate::ids::TEXPAT_SLOTS {
        for (k, seta, scrub, fabrica) in [
            (K::OffsetNum, 1.0, 1.0, Some(2.0)),
            (K::WidthNum, PASSO_FINO, PASSO_FINO, None),
            (K::HeightNum, PASSO_FINO, PASSO_FINO, None),
            (K::GapNum, SETA_DO_VAO, PASSO_FINO, Some(0.0)),
            (K::GapYNum, SETA_DO_VAO, PASSO_FINO, Some(0.0)),
            (K::ShiftXNum, 1.0, 1.0, Some(0.0)),
            (K::ShiftYNum, 1.0, 1.0, Some(0.0)),
            (K::AngleNum, 1.0, 1.0, Some(0.0)),
        ] {
            store.mark_card_field(kid(slot, k), spec(seta, scrub, fabrica));
        }
        let Some(p) = state::current_texture_pattern(slot) else {
            continue;
        };
        for (k, valor) in [
            (K::OffsetNum, p.offset_denom),
            (K::WidthNum, p.size[0]),
            (K::HeightNum, p.size[1]),
            (K::GapNum, p.gap[0]),
            (K::GapYNum, p.gap[1]),
            (K::ShiftXNum, p.shift_pct[0]),
            (K::ShiftYNum, p.shift_pct[1]),
            (K::AngleNum, p.angle_deg),
        ] {
            semeia(store, kid(slot, k), valor, false);
        }
    }
}

pub(crate) fn pattern_fill(c: &mut CardCtx<'_>, y: f32) -> f32 {
    pattern(c, 0, y)
}

pub(crate) fn pattern_stroke(c: &mut CardCtx<'_>, y: f32) -> f32 {
    pattern(c, 1, y)
}

/// O par `(rótulo, chip, barra)` de `fila`.
fn linha<'a>(nome: &'a str, slot: usize, num: K, barra: K) -> (&'a str, NodeId, NodeId) {
    (nome, kid(slot, num), kid(slot, barra))
}

/// Compacto como o Figma (05 §4): a fileira da arte, o reticulado, e os números aos pares numa grelha.
/// O Lock e o Link ficam lado a lado por baixo dos dois pares que governam.
fn pattern(c: &mut CardCtx<'_>, slot: usize, y: f32) -> f32 {
    let Some(p) = state::current_texture_pattern(slot) else {
        return y;
    };
    let metros = tr("card.vector.unit.metres");
    // O problema mesmo acima dos dois gestos que o resolvem.
    let mut y = match p.art {
        ph2d_vec_pattern::PatternArt::Ready => y,
        ph2d_vec_pattern::PatternArt::NotChosen => {
            c.hint(tr("panel.vector.texpat.art_not_chosen.hint"), y)
        }
        ph2d_vec_pattern::PatternArt::Deleted => {
            c.hint(tr("panel.vector.texpat.art_missing.hint"), y)
        }
    };
    y = par_de_botoes(
        c,
        (kid(slot, K::Source), tr("panel.vector.texpat.source")),
        (kid(slot, K::PickShape), crate::art_vocabulary::USE.tr()),
        y,
    );
    let tiles: Vec<_> = TILES
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let id = kid(slot, K::Tile(i as u8));
            seg(id, l.tr(), usize::from(p.kind) == i)
        })
        .collect();
    let canal: Vec<NodeId> = tiles.iter().map(|s| s.id).collect();
    y = escolha(c, tr("panel.vector.texpat.tile"), &canal, &tiles, y);
    let desfasa = com_unidade(
        tr("panel.vector.texpat.offset"),
        tr("card.vector.unit.one_over_n"),
    );
    let angulo = com_unidade(
        tr("panel.vector.texpat.angle"),
        tr("card.vector.unit.degrees"),
    );
    // O desfasamento só existe em Brick e Column: na grade não tem sentido, na colmeia é fixo.
    let mut topo = Vec::with_capacity(2);
    if matches!(p.kind, 1 | 2) {
        topo.push(linha(&desfasa, slot, K::OffsetNum, K::Offset));
    }
    topo.push(linha(&angulo, slot, K::AngleNum, K::Angle));
    y = fila(c, &topo, y);
    let (largura, altura) = (
        com_unidade(tr("panel.vector.texpat.width"), metros),
        com_unidade(tr("panel.vector.texpat.height"), metros),
    );
    y = fila(
        c,
        &[
            linha(&largura, slot, K::WidthNum, K::Width),
            linha(&altura, slot, K::HeightNum, K::Height),
        ],
        y,
    );
    let (vao_x, vao_y) = (
        com_unidade(tr("panel.vector.texpat.gap_x"), metros),
        com_unidade(tr("panel.vector.texpat.gap_y"), metros),
    );
    y = fila(
        c,
        &[
            linha(&vao_x, slot, K::GapNum, K::Gap),
            linha(&vao_y, slot, K::GapYNum, K::GapY),
        ],
        y,
    );
    // Ajuste de sessão, sem canal: o corpo de antes também não os marcava. Descem se a coluna é estreita.
    y = c.pair(
        caixa(
            kid(slot, K::Lock),
            Rotulo::vista(TextKey::new("panel.vector.texpat.lock")),
            p.lock_aspect,
        ),
        caixa(
            kid(slot, K::GapLink),
            Rotulo::vista(TextKey::new("panel.vector.texpat.gap_link")),
            p.link_gap,
        ),
        y,
    );
    if p.mode != CLAMP {
        let por_cento = tr("card.vector.unit.percent");
        let (fase_x, fase_y) = (
            com_unidade(tr("panel.vector.texpat.shift_x"), por_cento),
            com_unidade(tr("panel.vector.texpat.shift_y"), por_cento),
        );
        y = fila(
            c,
            &[
                linha(&fase_x, slot, K::ShiftXNum, K::ShiftX),
                linha(&fase_y, slot, K::ShiftYNum, K::ShiftY),
            ],
            y,
        );
    }
    let modos: Vec<_> = MODES
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let id = kid(slot, K::Mode(i as u8));
            seg(id, l.tr(), usize::from(p.mode) == i)
        })
        .collect();
    let canal: Vec<NodeId> = modos.iter().map(|s| s.id).collect();
    y = escolha(c, tr("panel.vector.texpat.repeat"), &canal, &modos, y);
    // A costura só no `Tile`: no Mirror ela fecha por construção, e o Mirror é o remédio.
    if p.mode == 0 && p.wrap_seam_visible {
        y = c.hint(tr("panel.vector.texpat.seam.hint"), y);
    }
    y
}
