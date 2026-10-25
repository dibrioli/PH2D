//! ⭐ **A coluna da esquerda do Pixel Sprite ▸ Draw** (dono 25/10: *«o padrão a seguir para a UI é a
//! mesma do módulo painter: tools na barra lateral esquerda»*): as quatro ferramentas e, em baixo, as
//! duas cores e o trocar — a MESMA porta das barras do vetor e do pintor ([`BarButton`]). Os ids são
//! os deste painel e o clique segue pelo despacho dele ([`crate::seam`]); as amostras abrem o seletor
//! partilhado num Down (`register_picker_swatch`).

use crate::ids;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily};
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelTool};
use ph2d_i18n::{tr, tr_with};

/// O ícone de cada ferramenta.
#[must_use]
pub const fn icon(t: PixelTool) -> IconId {
    match t {
        PixelTool::Pencil => IconId::VectorPencil,
        PixelTool::Eraser => IconId::Erase,
        PixelTool::Eyedropper => IconId::Eyedropper,
        PixelTool::Hand => IconId::Pan,
    }
}

/// O nome com o atalho: «Pencil (B)».
fn nome(t: PixelTool) -> String {
    tr_with(
        "pixel.tool_switched",
        &[("tool", &tr(t.label_key())), ("key", &t.shortcut())],
    )
}

/// Uma amostra cheia (o anel é o «traço» do vetor; aqui frente e fundo são duas cores iguais em
/// espécie, a de cima é a frente).
fn amostra(id: ph2d_a11y::NodeId, rgba: [u8; 4], key: &str) -> BarButton {
    BarButton {
        id,
        face: BarFace::Swatch {
            rgba: Some(rgba),
            ring: false,
            mixed: false,
        },
        label: tr(key).to_owned(),
        on: false,
        family: BarFamily::Neutral,
    }
}

/// **Os grupos da coluna** — vazio sem sprite em Draw (nenhum controlo morto).
#[must_use]
pub fn column(info: &PixelPanelInfo) -> Vec<Vec<BarButton>> {
    if !info.editing {
        return Vec::new();
    }
    let ferramentas = ids::TOOLS
        .iter()
        .map(|&(t, id)| BarButton {
            id,
            face: BarFace::Icon(icon(t)),
            label: nome(t),
            on: t == info.tool,
            family: BarFamily::Draw,
        })
        .collect();
    let cores = vec![
        amostra(ids::PIXEL_FG, info.fg, "panel.pixel.fg"),
        amostra(ids::PIXEL_BG, info.bg, "panel.pixel.bg"),
        BarButton {
            id: ids::PIXEL_SWAP,
            face: BarFace::Icon(IconId::PathReverse),
            label: tr("panel.pixel.swap_tip").to_owned(),
            on: false,
            family: BarFamily::Neutral,
        },
    ];
    vec![ferramentas, cores]
}

/// ⑤ A dica do rodapé: a ferramenta na mão, o que o arrasto faz.
#[must_use]
pub fn hint(info: &PixelPanelInfo) -> String {
    let key = match info.tool {
        PixelTool::Pencil => "pixel.hint.pencil",
        PixelTool::Eraser => "pixel.hint.eraser",
        PixelTool::Eyedropper => "pixel.hint.eyedropper",
        PixelTool::Hand => "pixel.hint.hand",
    };
    tr_with(key, &[("tool", &nome(info.tool))])
}
