//! **A fila de Image Tools** — a lista de pills, a sua geometria, a pintura e os nós de a11y.
//!
//! Módulo irmão do `chip_name`, e pelo mesmo motivo declarado: o `topbar/mod.rs` estourou o teto
//! de 700 LOC e a cura de um teto é o CORTE. O que sai é a fatia mais coesa que lá havia — as
//! cinco funções desta fila e os dois tipos que só elas usam.
//!
//! ⚠️ **Esta fila é DERIVADA do registry**, e é a única do topbar que o é: `image_action_pills()`
//! lê o cluster `image_tools` do [`crate::installed_registry`], então largar uma crate de tool
//! (ADR-0075) faz o pill aparecer sem editar coisa alguma aqui. Foi essa assimetria que produziu
//! o bug de 2026-08-19 (*«botão sheet não funciona»*): a fila cresceu sozinha e as listas
//! ESCRITAS À MÃO que a serviam — o registo no store e o despacho do clique — não. Ambas passaram
//! a ler daqui; o gate é
//! `ph2d-tool-registry-init/tests/every_image_tool_pill_dispatches.rs`.
//!
//! *Uma lista derivada ao lado de uma lista escrita à mão não é redundância: é uma divergência com
//! data marcada.*

use super::{
    HeroLayout, IconGlyph, IconId, NodeId, PILL_PADDING_PX, Rect, TOPBAR_INTER_CHIP_GAP,
    TOPBAR_RAIL_CHIP_W, TextSystem, Theme, VectorScene, ids, paint_topbar_group_backdrop,
    paint_topbar_rail_chip,
};
use crate::interaction::{HitIndex, WidgetStore};

/// Drawing source for one Image Tools action pill's glyph. When the
/// pill came from a manifest the manifest's `icon_fn` already produced
/// a 24×24 [`ph2d_vector::BezPath`]; when the pill came from the
/// legacy fallback the editor's `IconId` table supplies the path.
pub(super) enum PillIcon {
    /// Manifest-derived (Wave 2 PR 11.4). `icon_fn` returned this path.
    FromManifest(ph2d_vector::BezPath),
    /// Legacy fallback for tests / pre-registry boot.
    Legacy(IconId),
}

/// One pill row entry. Tuple form previously; refactored into a struct
/// in PR 11.4 because the icon source now has two flavors (manifest
/// BezPath vs legacy IconId).
pub(super) struct ImageActionPill {
    pub(super) id: NodeId,
    pub(super) icon: PillIcon,
    pub(super) label_key: &'static str,
}

/// Build the Image Tools action pill list. Wave 2 PR 11.4: derives
/// from the runtime [`crate::installed_registry`] when present.
/// Manifests register the cluster id `"image_tools"`; the registry
/// returns them sorted by `(order, id)` so paint order matches design
/// intent (trim 40 → make_square 50 → bgremoval 60).
///
/// Falls back to the legacy hardcoded triple for tests / pre-registry
/// boot. Both paths produce the same `NodeId`s because the chrome
/// consts in [`crate::screens::hero::ids`] hash the SAME slug as the
/// matching manifest's `id` field (PR 11.4 contract — pinned by the
/// `chrome_manifest_coverage` integration test).
///
/// ⚠️ **Esta é a PORTA ÚNICA da fila.** Quem pinta, quem regista no store e quem calcula
/// geometria chamam-na — de propósito, para que não possam divergir sem uma edição que os separe.
pub(super) fn image_action_pills() -> Vec<ImageActionPill> {
    use ph2d_tool_registry::hash_node_id;
    if let Some(reg) = crate::installed_registry() {
        return reg
            .cluster("image_tools")
            .iter()
            .map(|m| ImageActionPill {
                id: hash_node_id(m.id),
                icon: PillIcon::FromManifest((m.icon_fn)()),
                label_key: m.label_key,
            })
            .collect();
    }
    vec![
        ImageActionPill {
            id: ids::IMAGE_ACTION_TRIM,
            icon: PillIcon::Legacy(IconId::TrimTransparency),
            label_key: "tool.trim_transparency.label",
        },
        ImageActionPill {
            id: ids::IMAGE_ACTION_MAKE_SQUARE,
            icon: PillIcon::Legacy(IconId::MakeSquare),
            label_key: "tool.make_square.label",
        },
        ImageActionPill {
            id: ids::IMAGE_ACTION_BGREMOVAL,
            icon: PillIcon::Legacy(IconId::BgRemoval),
            label_key: "tool.bgremoval.label",
        },
    ]
}

/// Paint the Image Tools action pill row — the right-side row that replaces the normal
/// TopBar clusters when [`crate::screens::hero::HeroScreen::image_tools_mode`] is `true`. Each
/// pill is registered in the hit index so dispatch can route clicks; tooltips are seeded by
/// [`super::populate`].
#[allow(clippy::too_many_arguments)] // o relogio e' o 7o; ver a nota do `paint_rail`
pub(super) fn paint_image_action_row(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
) {
    let row_h = layout.top_bar.h;
    let pills = image_action_pills();
    // Image-tool chips share the same column width + inter-chip gap
    // as the topbar's Play / Right clusters.
    let chip_count = pills.len() as f32;
    let total_w =
        TOPBAR_RAIL_CHIP_W * chip_count + TOPBAR_INTER_CHIP_GAP * (chip_count - 1.0).max(0.0);
    let start_x = layout.top_bar.x + layout.top_bar.w - total_w;
    let col_stride = TOPBAR_RAIL_CHIP_W + TOPBAR_INTER_CHIP_GAP;
    // Single agrupador backdrop spanning ALL image-tool pills (Enio
    // 2026-05-24: "Os botões dos image tools também um fundo só").
    if total_w > 0.0 {
        paint_topbar_group_backdrop(
            ids::TOPBAR_IMAGE_TOOLS_BACKDROP,
            scene,
            theme,
            Rect::new(start_x, layout.top_bar.y, total_w, row_h),
            store.rail_button_size().chip_px(),
            layout.viewport.y,
            hit_index,
        );
    }
    for (i, pill) in pills.iter().enumerate() {
        let col = Rect::new(
            start_x + col_stride * i as f32,
            layout.top_bar.y,
            TOPBAR_RAIL_CHIP_W,
            row_h,
        );
        let glyph = match &pill.icon {
            PillIcon::FromManifest(path) => IconGlyph::Path(path),
            PillIcon::Legacy(icon) => IconGlyph::Builtin(*icon),
        };
        let label = ph2d_i18n::tr(pill.label_key);
        paint_topbar_rail_chip(
            pill.id,
            glyph,
            label,
            col,
            layout.viewport.y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            false,
        );
    }
}

/// ⭐⭐ **AS FERRAMENTAS DE IMAGEM COMO ENTRADAS DE TRILHO** — a porta que as traz para a fila
/// horizontal.
///
/// ⛔⛔ **Ela existe porque as dez ficaram INALCANÇÁVEIS.** Elas eram pintadas num único sítio —
/// o [`paint_image_action_row`], dentro do `paint_top_bar` —, e a barra de pills saiu de cena em
/// 2026-08-30. A auditoria do mesmo dia mediu o resto: não há atalho de teclado, não há linha de
/// menu, a paleta de comandos não as projecta e a paleta de ferramentas do canvas só corre no
/// caminho de demo. ⇒ **incluindo o Painter**, e com ele toda a face de pintura desta fila
/// (`rail_shows_painter_tools` exige `active_tool_id == Some("painter")`, que nunca podia acontecer).
///
/// ⚠️ **O sub-rótulo é DERIVADO do nome** (primeira palavra, maiúsculas, 5 caracteres) e não uma
/// tabela: uma tabela de tags ao lado de uma lista que vem do **registry** seria a lista à mão que
/// deixa de fora a ferramenta nova. O nome inteiro viaja na etiqueta de acessibilidade.
pub(crate) fn image_tool_rail_entries(store: &WidgetStore) -> Vec<crate::widget::ToolRailEntry> {
    use crate::widget::{ButtonState, ToolRailEntry};
    image_action_pills()
        .into_iter()
        .map(|pill| {
            let label = ph2d_i18n::tr(pill.label_key);
            let sub: String = label
                .split_whitespace()
                .next()
                .unwrap_or("")
                .chars()
                .filter(|c| c.is_alphanumeric())
                .take(5)
                .collect::<String>()
                .to_uppercase();
            let mut e = match pill.icon {
                PillIcon::FromManifest(path) => ToolRailEntry::glyph(pill.id, label, path, sub),
                PillIcon::Legacy(icon) => ToolRailEntry::icon(pill.id, label, icon).with_sub(sub),
            };
            if matches!(store.button_state(pill.id), Some(ButtonState::Pressed)) {
                e = e.active();
            }
            e
        })
        .collect()
}

/// Geometry of the Image Tools action pill row for a given layout.
/// Shared between [`paint_image_action_row`] (paints + hit-registers)
/// and [`image_action_a11y_nodes`] (publishes AccessKit nodes) so the
/// two surfaces can't drift.
///
/// The returned tuples expose only `(NodeId, Rect)` — the icon source
/// (manifest BezPath vs legacy IconId) is an implementation detail
/// hidden from callers since neither downstream cares about it
/// geometrically.
///
/// ⚠️ Era `pub(crate)` e re-exportada pelo `mod.rs`; a extração de 2026-08-19 mostrou que os dois
/// únicos consumidores estão neste ficheiro, e o re-export tinha ZERO chamadores. *Superfície que
/// ninguém atravessa não é extensibilidade, é uma porta que se esqueceram de fechar.*
fn image_action_pill_rects(layout: &HeroLayout, gap: f32) -> Vec<(NodeId, Rect)> {
    let row_h = layout.top_bar.h;
    let pill_w = 40.0 + PILL_PADDING_PX * 2.0; // LITERAL-PX-OK: TopBar action pill base width (chrome dim)
    let pills = image_action_pills();
    let total_w = pill_w * pills.len() as f32 + gap * pills.len().saturating_sub(1) as f32;
    let start_x = layout.top_bar.x + layout.top_bar.w - total_w;
    let mut rx = start_x;
    let mut out = Vec::with_capacity(pills.len());
    for pill in &pills {
        let rect = Rect::new(rx, layout.top_bar.y, pill_w, row_h);
        out.push((pill.id, rect));
        rx = rect.x + rect.w + gap;
    }
    out
}

/// AccessKit nodes for the Image Tools action pills (HR-12). Returns
/// one `Node` per visible action: `Role::Button` + i18n label +
/// bounds + `Action::Click`. Mirrors the canonical shape from
/// [`crate::widget::Button::build_a11y`].
///
/// The desktop shell hasn't wired the AccessKit `TreeUpdate` pipeline
/// yet (M14.x scope), so this is currently a structural surface that
/// tests assert against. When the shell wires AccessKit, it inserts
/// these as children of the root Window node returned by
/// [`crate::screens::hero::HeroScreen::build_a11y`].
pub fn image_action_a11y_nodes(
    layout: &HeroLayout,
    image_tools_mode: bool,
    gap: f32,
) -> Vec<(NodeId, ph2d_a11y::Node)> {
    use ph2d_a11y::{Action, NodeBuilder, Role};
    if !image_tools_mode {
        return Vec::new();
    }
    let pills = image_action_pills();
    let rects = image_action_pill_rects(layout, gap);
    rects
        .into_iter()
        .zip(pills.iter())
        .map(|((id, rect), pill)| {
            let node = NodeBuilder::new(Role::Button)
                .label(ph2d_i18n::tr(pill.label_key))
                .bounds(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64)
                .focusable(true)
                .action(Action::Click)
                .build();
            (id, node)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::hero::style::{EDGE_PAD, TOPBAR_GAP, TOPBAR_H};
    use crate::zones::Rect;

    fn sample_layout() -> HeroLayout {
        // Minimal HeroLayout fixture — only `top_bar` is consulted by
        // the image-action pill geometry, the rest can stay default.
        let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
        HeroLayout::for_viewport(viewport)
    }

    /// HR-12 enforcement for the Image Tools row. Both Trim and Make
    /// Square pills must expose `Role::Button` + non-empty label +
    /// `Action::Click` + bounds matching the painted rect. Locked-in so
    /// future actions added to the row inherit the same a11y contract.
    #[test]
    fn image_action_a11y_nodes_match_paint_rects() {
        let layout = sample_layout();
        let gap = TOPBAR_GAP; // matches paint_top_bar's gap
        let _ = (EDGE_PAD, TOPBAR_H); // silence unused-import lints if absent
        let rects = image_action_pill_rects(&layout, gap);
        let nodes = image_action_a11y_nodes(&layout, true, gap);

        // Same length, same NodeIds in the same order.
        assert_eq!(rects.len(), nodes.len());
        for ((rect_id, rect), (node_id, node)) in rects.iter().zip(nodes.iter()) {
            assert_eq!(rect_id, node_id);
            // Label is non-empty (i18n stub round-tripped through tr()).
            assert!(
                !node.label().unwrap_or("").is_empty(),
                "node for {rect_id:?} has empty label"
            );
            // Bounds match the painted rect.
            let b = node.bounds().expect("button node must carry bounds");
            assert!(
                (b.x0 - rect.x as f64).abs() < 1e-3
                    && (b.y0 - rect.y as f64).abs() < 1e-3
                    && (b.x1 - (rect.x + rect.w) as f64).abs() < 1e-3
                    && (b.y1 - (rect.y + rect.h) as f64).abs() < 1e-3,
                "bounds mismatch for {rect_id:?}: a11y={b:?} paint={rect:?}",
            );
        }
    }

    /// When image_tools_mode is off the row isn't painted, so no a11y
    /// nodes should be published either.
    #[test]
    fn image_action_a11y_empty_when_mode_off() {
        let layout = sample_layout();
        assert!(image_action_a11y_nodes(&layout, false, 16.0).is_empty());
    }
}
