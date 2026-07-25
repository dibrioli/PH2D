//! **A cena pronta para o smoke do WARP** (`PH2D_BUILD_SMOKE=26`) — a família de efeitos
//! paramétricos do menu *Effect > Warp* do Illustrator (Arc/Bulge/Wave/Fisheye/Rise).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `text_fx_smoke`.
//!
//! # As duas metades, a mesma cicatriz do impasto_smoke
//!
//! *"nothing here is armed in code … the smoke that arms state under the table skips exactly the
//! seam it was supposed to prove"*. Mas armar NADA deixa o artista sem referência do que a coisa
//! deve parecer. A resposta é a mesma das cenas de Contour/física: **uma forma PELADA para autorar
//! pela UI, e as PRONTAS para julgar o desenho**.
//!
//! - **cinco retângulos**, um por estilo, cada um com o Warp ARMADO numa dobra forte: é o
//!   render-and-look — um warp que a matemática produz feio aparece aqui de imediato;
//! - **um retângulo PELADO**, selecionado: é ele que prova o seam (a seção Effects → menu Add →
//!   os cinco estilos novos → o slider Bend).

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_warp_presets::{WarpSpec, WarpStyle};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecVertex};

/// A dobra dos exemplos, em percentagem. Forte de propósito: um warp discreto não é um teste.
const DEMO_BEND: f64 = 55.0;

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => arm(app),
        _ => {}
    }
}

/// Um retângulo fechado de `w × h` centrado em `(cx, cy)`, preenchido.
fn rect(cx: f64, cy: f64, w: f64, h: f64, rgb: [u8; 3]) -> VecPath {
    let (hw, hh) = (w * 0.5, h * 0.5);
    VecPath {
        verts: [
            [cx - hw, cy - hh],
            [cx + hw, cy - hh],
            [cx + hw, cy + hh],
            [cx - hw, cy + hh],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255))),
        ..VecPath::default()
    }
}

/// Monta os cinco retângulos armados (um por estilo) + o pelado, e guarda o id do pelado.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    // Uma fileira dos cinco estilos, do canto esquerdo para a direita.
    let n = WarpStyle::ALL.len();
    #[allow(clippy::cast_precision_loss)]
    let span = 1.5_f64;
    #[allow(clippy::cast_precision_loss)]
    let x0 = -((n as f64) - 1.0) * 0.5 * span;
    for (i, &style) in WarpStyle::ALL.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let cx = x0 + (i as f64) * span;
        let id = scene.push_path(rect(cx, 1.1, 1.1, 0.8, [90, 150, 220]));
        if let Some(p) = scene.path_mut(id) {
            // Só a dobra nos cinco demos, para cada estilo ler LIMPO; as duas perspectivas
            // (Horizontal/Vertical) o artista experimenta no retângulo pelado.
            p.effects = vec![FxEntry::new(PathEffect::Warp(WarpSpec {
                style,
                bend: DEMO_BEND,
                h_distort: 0.0,
                v_distort: 0.0,
            }))];
        }
    }

    // O retângulo PELADO, embaixo — a metade que se autora pela UI.
    let pelado = scene.push_path(rect(0.0, -1.3, 1.6, 1.0, [235, 175, 60]));
    PENDING.lock().expect("smoke lock").replace(pelado);
}

/// O id do retângulo pelado, à espera da entidade que o `sync` lhe dá no frame seguinte.
static PENDING: std::sync::Mutex<Option<VecPathId>> = std::sync::Mutex::new(None);

/// Seleciona o retângulo pelado e imprime o roteiro.
fn arm(app: &mut crate::App) {
    let Some(id) = PENDING.lock().expect("smoke lock").take() else {
        return;
    };
    app.vec_pen.select_many(&[id]);
    let names: Vec<&str> = WarpStyle::ALL.iter().map(|s| s.label()).collect();
    eprintln!(
        "[smoke] WARP -- a familia Effect > Warp (menu Add da secao Effects).\n\
         [smoke]   A fileira de CIMA: cinco retangulos azuis, um por estilo ({}), cada um com\n\
         [smoke]   o Warp armado a Bend={DEMO_BEND}. E o render-and-look: cada silhueta tem de\n\
         [smoke]   deformar SUAVE (as bordas viram curvas, nao facetas) e reconhecivel --\n\
         [smoke]   Arc arqueia, Wave faz um S, Bulge/Fisheye abaulam, Rise inclina.\n\
         [smoke]   O retangulo LARANJA embaixo esta pelado e selecionado:\n\
         [smoke]   1. Abra a secao **Effects** no painel. O menu Add tem de listar os cinco\n\
         [smoke]      estilos novos alem dos quatro velhos (Trim/Zig Zag/Repeater/Pucker&Bloat).\n\
         [smoke]   2. Clique **Add Arc** (ou outro). A forma deforma NA HORA (nasce neutra: o\n\
         [smoke]      clique nao move um pixel ate voce arrastar um slider).\n\
         [smoke]   3. Sao TRES sliders, como o dialogo Warp do Illustrator: **Bend** dobra;\n\
         [smoke]      **Horizontal** e **Vertical** dao a PERSPECTIVA (o keystone -- uma borda\n\
         [smoke]      alarga contra a oposta). Arraste os tres, e os dois lados espelham; H/V\n\
         [smoke]      compoem com a dobra (Arc + Horizontal = um arco em fuga).\n\
         [smoke]   4. Empilhe: Add um 2o warp por cima; a ordem importa (Arc-depois-Wave nao e\n\
         [smoke]      Wave-depois-Arc). **Apply** assa a pilha na geometria.",
        names.join(" / ")
    );
}
