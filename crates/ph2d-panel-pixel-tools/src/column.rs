//! ⭐ **A coluna da esquerda do Pixel Sprite ▸ Draw** (dono 25/10: *«o padrão a seguir para a UI é a
//! mesma do módulo painter: tools na barra lateral esquerda»*): as ferramentas em três grupos e, em baixo, as
//! duas cores e o trocar — a MESMA porta das barras do vetor e do pintor ([`BarButton`]). Os ids são
//! os deste painel e o clique segue pelo despacho dele ([`crate::seam`]); as amostras abrem o seletor
//! partilhado num Down (`register_picker_swatch`).

use crate::ids;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily};
use ph2d_editor_core::pixel_panel::{PixelInk, PixelPanelInfo, PixelTool};
use ph2d_i18n::{tr, tr_with};

/// O ícone de cada ferramenta.
#[must_use]
pub const fn icon(t: PixelTool) -> IconId {
    match t {
        PixelTool::RectMarquee => IconId::MarqueeBox,
        PixelTool::EllipseMarquee => IconId::Circle,
        PixelTool::Lasso => IconId::MarqueeLasso,
        PixelTool::PolygonLasso => IconId::VectorPen,
        PixelTool::MagicWand => IconId::Mask,
        PixelTool::Move => IconId::Transform,
        PixelTool::Pencil => IconId::VectorPencil,
        PixelTool::Spray => IconId::PixelSpray,
        PixelTool::Eraser => IconId::Erase,
        PixelTool::Eyedropper => IconId::Eyedropper,
        PixelTool::Bucket => IconId::VectorBucket,
        PixelTool::Line => IconId::Line,
        PixelTool::Curve => IconId::PixelCurve,
        PixelTool::Rectangle => IconId::PixelRect,
        PixelTool::FilledRectangle => IconId::PixelRectFilled,
        PixelTool::Ellipse => IconId::PixelEllipse,
        PixelTool::FilledEllipse => IconId::PixelEllipseFilled,
        PixelTool::Contour => IconId::PixelContour,
        PixelTool::Polygon => IconId::Polygon,
        PixelTool::Hand => IconId::Pan,
    }
}

/// O nome com o atalho: «Pencil (B)».
fn nome(t: PixelTool) -> String {
    tr_with(
        "pixel.tool_switched",
        &[("tool", &tr(t.label_key())), ("key", &t.shortcut_label())],
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
    // Quatro grupos, como a barra do Aseprite: a selecção (e o Move), o que pinta à mão, as
    // formas, a vista.
    let grupo = |de: PixelTool, ate: PixelTool| {
        ids::TOOLS
            .iter()
            .skip_while(|(t, _)| *t != de)
            .take_while(|(t, _)| *t != ate)
            .map(|&(t, id)| BarButton {
                id,
                face: BarFace::Icon(icon(t)),
                label: nome(t),
                on: t == info.tool,
                family: BarFamily::Draw,
            })
            .collect::<Vec<_>>()
    };
    let selecao = grupo(PixelTool::RectMarquee, PixelTool::Pencil);
    let mao = grupo(PixelTool::Pencil, PixelTool::Line);
    let formas = grupo(PixelTool::Line, PixelTool::Hand);
    let vista = grupo(PixelTool::Hand, PixelTool::RectMarquee);
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
    vec![selecao, mao, formas, vista, cores]
}

/// ⑤ A dica do rodapé: a ferramenta na mão, o que o arrasto faz.
#[must_use]
pub fn hint(info: &PixelPanelInfo) -> String {
    // Com o Shading o botão direito ESCURECE (não pinta a cor de fundo): a dica diz o que o gesto faz.
    let shading = info.options.ink == PixelInk::Shading
        && info.tool.paints()
        && info.tool != PixelTool::Eraser;
    let key = match info.tool {
        _ if shading => "pixel.hint.shading",
        PixelTool::RectMarquee | PixelTool::EllipseMarquee => "pixel.hint.marquee",
        PixelTool::Lasso => "pixel.hint.lasso",
        PixelTool::PolygonLasso => "pixel.hint.polygon_lasso",
        PixelTool::MagicWand => "pixel.hint.magic_wand",
        PixelTool::Move => "pixel.hint.move",
        PixelTool::Pencil => "pixel.hint.pencil",
        PixelTool::Spray => "pixel.hint.spray",
        PixelTool::Eraser => "pixel.hint.eraser",
        PixelTool::Eyedropper => "pixel.hint.eyedropper",
        PixelTool::Bucket => "pixel.hint.bucket",
        PixelTool::Line => "pixel.hint.line",
        PixelTool::Curve => "pixel.hint.curve",
        PixelTool::Rectangle | PixelTool::FilledRectangle => "pixel.hint.rectangle",
        PixelTool::Ellipse | PixelTool::FilledEllipse => "pixel.hint.ellipse",
        PixelTool::Contour => "pixel.hint.contour",
        PixelTool::Polygon => "pixel.hint.polygon",
        PixelTool::Hand => "pixel.hint.hand",
    };
    tr_with(key, &[("tool", &nome(info.tool))])
}
