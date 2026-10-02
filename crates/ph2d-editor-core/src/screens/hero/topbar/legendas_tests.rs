//! Gate das LEGENDAS da barra do topo (o chrome legado, `F9`) — a mesma lei da fila de ferramentas.

use super::cluster_painter::{
    TOPBAR_INTER_CHIP_GAP, TOPBAR_RAIL_CHIP_W, cluster_width, paint_top_bar_cluster,
    paint_topbar_rail_chip,
};
use super::image_action_row::{PillIcon, image_action_pills};
use crate::interaction::{HitIndex, WidgetStore};
use crate::screens::hero::fixture::topbar_clusters;
use crate::screens::hero::menu_bar::tests::em_todo_estilo;
use crate::widget::IconGlyph;
use crate::widget::RailButtonSize;
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⭐⭐ **Nada do que a barra do topo escreve sai cortado** (2026-10-02) — em toda fonte, peso,
/// tamanho de texto, nitidez e tamanho de botão, sobre TODOS os grupos da fixtura (os avulsos e os
/// de três colunas). As legendas eram medidas como texto DENTRO de uma moldura (`44 − 16 = 28 px`)
/// e não têm moldura nenhuma. Régua: o que foi PINTADO.
#[test]
fn nenhuma_legenda_da_barra_do_topo_e_cortada() {
    let clusters = topbar_clusters();
    assert!(
        clusters.len() > 5,
        "a barra do topo perdeu os grupos — fixtura vazia"
    );
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        for size in [
            RailButtonSize::Small,
            RailButtonSize::Medium,
            RailButtonSize::Large,
        ] {
            let mut store = WidgetStore::default();
            store.set_rail_button_size(size);
            let motion = crate::motion::UiMotion::default();
            for (id, cluster) in &clusters {
                let mut scene = VectorScene::new();
                let mut hit = HitIndex::new();
                let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                    paint_top_bar_cluster(
                        *id,
                        cluster,
                        Rect::new(0.0, 0.0, cluster_width(cluster), 64.0),
                        0.0,
                        &mut scene,
                        ts,
                        Theme::default(),
                        &mut hit,
                        &store,
                        &motion,
                        false,
                    );
                });
                fora.extend(medidos.iter().filter(|m| !m.coube()).map(|m| {
                    format!(
                        "{size:?}: «{}» -> «{}» ({:.1} px)",
                        m.texto, m.pintado, m.largura
                    )
                }));
            }
            // A fileira das ferramentas de imagem (o modo Image Tools) usa o mesmo chip.
            for pill in image_action_pills() {
                let glyph = match &pill.icon {
                    PillIcon::FromManifest(path) => IconGlyph::Path(path),
                    PillIcon::Legacy(icon) => IconGlyph::Builtin(*icon),
                };
                let mut scene = VectorScene::new();
                let mut hit = HitIndex::new();
                let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                    paint_topbar_rail_chip(
                        pill.id,
                        glyph,
                        ph2d_i18n::tr(pill.label_key),
                        Rect::new(0.0, 0.0, TOPBAR_RAIL_CHIP_W, 64.0),
                        TOPBAR_INTER_CHIP_GAP,
                        0.0,
                        &mut scene,
                        ts,
                        Theme::default(),
                        &mut hit,
                        &store,
                        &motion,
                        false,
                    );
                });
                fora.extend(medidos.iter().filter(|m| !m.coube()).map(|m| {
                    format!(
                        "{size:?} imagem: «{}» -> «{}» ({:.1} px)",
                        m.texto, m.pintado, m.largura
                    )
                }));
            }
        }
        fora
    });
    assert!(
        acusados.is_empty(),
        "legendas cortadas ({}):\n  {}",
        acusados.len(),
        acusados
            .iter()
            .take(60)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
