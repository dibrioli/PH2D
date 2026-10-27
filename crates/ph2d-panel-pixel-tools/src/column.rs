//! ⭐ **A coluna da esquerda do Pixel Sprite ▸ Draw** (dono 25/10: *«o padrão a seguir para a UI é a
//! mesma do módulo painter: tools na barra lateral esquerda»*): as ferramentas em três grupos e, em baixo, as
//! duas cores e o trocar — a MESMA porta das barras do vetor e do pintor ([`BarButton`]). Os ids são
//! os deste painel e o clique segue pelo despacho dele ([`crate::seam`]); as amostras abrem o seletor
//! partilhado num Down (`register_picker_swatch`).

use crate::ids;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, ToolBars};
use ph2d_editor_core::pixel_panel::{PixelInk, PixelPanelInfo, PixelShape, PixelTool};
use ph2d_i18n::{tr, tr_with};

/// O ícone de cada ferramenta.
#[must_use]
pub const fn icon(t: PixelTool) -> IconId {
    match t {
        PixelTool::RectMarquee => IconId::MarqueeBox,
        PixelTool::EllipseMarquee => IconId::CircleDashed,
        PixelTool::Lasso => IconId::MarqueeLasso,
        PixelTool::PolygonLasso => IconId::PolygonLasso,
        PixelTool::MagicWand => IconId::WandSparkles,
        PixelTool::Transform => IconId::Transform,
        PixelTool::Pencil => IconId::VectorPencil,
        PixelTool::Spray => IconId::PixelSpray,
        PixelTool::Eraser => IconId::Erase,
        PixelTool::Eyedropper => IconId::Eyedropper,
        PixelTool::Bucket => IconId::VectorBucket,
        PixelTool::Line => IconId::Line,
        PixelTool::Curve => IconId::PixelCurve,
        PixelTool::Rectangle => IconId::PixelRect,
        PixelTool::Ellipse => IconId::PixelEllipse,
        PixelTool::Contour => IconId::PixelContour,
        PixelTool::Polygon => IconId::Polygon,
        PixelTool::Hand => IconId::Pan,
        // A cara delas é o CONTORNO do Vector ([`face`]); estes são só o nome do ícone.
        PixelTool::RegularPolygon | PixelTool::Star => IconId::VectorShape,
    }
}

/// A cara do botão: o ícone, ou — para as formas do Vector — o contorno que o `cook` do Vector
/// desenha num quadrado unitário (a lei da barra do Vector: a mesma geometria, nunca um 2.º desenho).
#[must_use]
pub fn face(t: PixelTool) -> BarFace {
    let forma = match t {
        PixelTool::RegularPolygon => PixelShape::Polygon,
        PixelTool::Star => PixelShape::Star,
        _ => return BarFace::Icon(icon(t)),
    };
    let k = forma.kind();
    let path = ph2d_vec_scene::cook(k, [0.0, 0.0], [1.0, 1.0], &k.defaults());
    let mut bez = ph2d_vector::BezPath::new();
    let p = |q: [f64; 2]| (q[0], 1.0 - q[1]);
    for c in ph2d_vec_scene::trim_tool::contours_of(&path) {
        let Some(first) = c.verts.first() else {
            continue;
        };
        bez.move_to(p(first.anchor));
        let n = c.verts.len();
        let arestas = if c.closed { n } else { n - 1 };
        for i in 0..arestas {
            let (a, b) = (&c.verts[i], &c.verts[(i + 1) % n]);
            bez.curve_to(p(a.out_handle), p(b.in_handle), p(b.anchor));
        }
        if c.closed {
            bez.close_path();
        }
    }
    BarFace::Outline(bez)
}

/// O nome com o atalho: «Pencil (B)».
fn nome(t: PixelTool) -> String {
    t.named()
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

/// O botão de uma ferramenta.
fn botao(info: &PixelPanelInfo, t: PixelTool, id: ph2d_a11y::NodeId) -> BarButton {
    BarButton {
        id,
        face: face(t),
        label: nome(t),
        on: t == info.tool,
        family: BarFamily::Draw,
    }
}

/// ⭐ **As barras do Pixel em Draw**: a coluna, o flyout do botão de grupo aberto (as selecções ou
/// as formas, dono 26/10) e a dica do rodapé.
#[must_use]
pub fn bars(info: &PixelPanelInfo) -> ToolBars {
    let column = column(info);
    let flyout = info
        .groups
        .open
        .filter(|_| info.editing)
        .and_then(|g| ids::GROUPS.iter().find(|(h, _)| *h == g))
        .map(|&(g, gid)| {
            let todas = g
                .members()
                .iter()
                .filter_map(|t| ids::TOOLS.iter().find(|(u, _)| u == t))
                .map(|&(t, id)| botao(info, t, id))
                .collect();
            (gid, todas)
        });
    ToolBars {
        column,
        flyout,
        grouped: ids::GROUPS.iter().map(|(_, id)| *id).collect(),
        hint: hint(info),
        ..ToolBars::default()
    }
}

/// **Os grupos da coluna** — vazio sem sprite em Draw (nenhum controlo morto).
#[must_use]
pub fn column(info: &PixelPanelInfo) -> Vec<Vec<BarButton>> {
    if !info.editing {
        return Vec::new();
    }
    // Quatro grupos, como a barra do Aseprite: a selecção (e o Transform), o que pinta à mão, as
    // formas, a vista. As de um botão de grupo moram nele, no lugar da 1.ª, com a cara da última.
    let grupo = |de: PixelTool, ate: PixelTool| {
        ids::TOOLS
            .iter()
            .skip_while(|(t, _)| *t != de)
            .take_while(|(t, _)| *t != ate)
            .filter_map(|&(t, id)| match t.group() {
                None => Some(botao(info, t, id)),
                Some(g) if g.members()[0] == t => {
                    let (_, gid) = ids::GROUPS.iter().find(|(h, _)| *h == g)?;
                    Some(BarButton {
                        on: info.tool.group() == Some(g),
                        ..botao(info, info.groups.face(g), *gid)
                    })
                }
                Some(_) => None,
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
        _ if info.live_shape => "pixel.hint.live_shape",
        _ if shading => "pixel.hint.shading",
        PixelTool::RectMarquee | PixelTool::EllipseMarquee => "pixel.hint.marquee",
        PixelTool::Lasso => "pixel.hint.lasso",
        PixelTool::PolygonLasso => "pixel.hint.polygon_lasso",
        PixelTool::MagicWand => "pixel.hint.magic_wand",
        PixelTool::Transform => "pixel.hint.transform",
        PixelTool::Pencil => "pixel.hint.pencil",
        PixelTool::Spray => "pixel.hint.spray",
        PixelTool::Eraser => "pixel.hint.eraser",
        PixelTool::Eyedropper => "pixel.hint.eyedropper",
        PixelTool::Bucket => "pixel.hint.bucket",
        PixelTool::Line => "pixel.hint.line",
        PixelTool::Curve => "pixel.hint.curve",
        PixelTool::Rectangle => "pixel.hint.rectangle",
        PixelTool::Ellipse => "pixel.hint.ellipse",
        PixelTool::Contour => "pixel.hint.contour",
        PixelTool::Polygon => "pixel.hint.polygon",
        PixelTool::RegularPolygon | PixelTool::Star => "pixel.hint.shape",
        PixelTool::Hand => "pixel.hint.hand",
    };
    tr_with(key, &[("tool", &nome(info.tool))])
}
