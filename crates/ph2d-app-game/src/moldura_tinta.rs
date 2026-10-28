//! A tinta da câmera no canvas (plano GEF 17 A1/A2/A4): o ícone no lugar do anel do vazio, a moldura (forte com a
//! câmera selecionada, ténue sem ela, tracejada se não é a que manda), o rótulo do tamanho, os cantos e os limites.
//! Em espaço de TELA: o traço e o texto ficam nítidos em qualquer zoom.

use crate::moldura::{Moldura, caixa, cantos};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{CameraFrameShow, icons::IconId};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::{Affine, Color, Fill, Stroke, VectorScene};

/// A opacidade da moldura de uma câmera não selecionada (ténue, D-V «Sempre»).
const TENUE: f32 = 0.45; // LITERAL-PX-OK: opacidade de overlay
const TRACO_PX: [f64; 2] = [1.0, 2.0]; // LITERAL-PX-OK: chrome de overlay (ténue, forte)
const TRACEJADO_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: chrome de overlay
const ALCA_PX: f64 = 4.0; // LITERAL-PX-OK: meia-alça
const ICONE_PX: f32 = 22.0; // LITERAL-PX-OK: lado do ícone da câmera
const ROTULO_FOLGA_PX: f32 = 4.0; // LITERAL-PX-OK: entre o rótulo e a moldura
const ROTULO_MAX_PX: f32 = 420.0; // LITERAL-PX-OK: largura máxima do rótulo

/// As molduras que se veem: todas com «Sempre», só a selecionada com «Só selecionada».
pub fn visiveis(ms: &[Moldura], mostrar: CameraFrameShow) -> impl Iterator<Item = &Moldura> {
    ms.iter()
        .filter(move |m| mostrar == CameraFrameShow::Always || m.selecionada)
}

/// Pinta a câmera no canvas.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(
    ms: &[Moldura],
    mostrar: CameraFrameShow,
    unidade: DisplayUnit,
    pixels_per_meter: f32,
    theme: Theme,
    camera: &Camera2d,
    window: WindowSize,
    cena: &mut VectorScene,
    texto: &mut TextSystem,
) {
    let selecao = ph2d_editor_core::paint::resolve(ColorToken::Selection, theme);
    let limites = ph2d_editor_core::paint::resolve(ColorToken::Warn, theme);
    let fundo = ph2d_editor_core::paint::resolve(ColorToken::Bg0, theme);
    let tela = |p: [f32; 2]| {
        let (x, y) = camera.world_to_screen(p, window);
        (f64::from(x), f64::from(y))
    };
    for m in ms {
        let (x, y) = tela(m.pose);
        let lado = f64::from(ICONE_PX);
        let r = Rect::new(
            (x - lado * 0.5) as f32,
            (y - lado * 0.5) as f32,
            ICONE_PX,
            ICONE_PX,
        );
        ph2d_editor_core::paint_icon(cena, IconId::Camera, r, selecao, TRACO_PX[1] as f32);
    }
    for m in visiveis(ms, mostrar) {
        let forte = m.selecionada;
        let cor = if forte {
            selecao
        } else {
            selecao.multiply_alpha(TENUE)
        };
        let (lo, hi) = caixa(m);
        let rect = retangulo(tela([lo[0], hi[1]]), tela([hi[0], lo[1]]));
        let traco = Stroke::new(TRACO_PX[usize::from(forte)]);
        let traco = if m.ativa {
            traco
        } else {
            traco.with_dashes(0.0, TRACEJADO_PX)
        };
        let s = cena.inner_mut();
        s.stroke(&traco, Affine::IDENTITY, cor, None, &rect);
        let w = unidade.from_meters(m.meia[0] * 2.0, pixels_per_meter);
        let h = unidade.from_meters(m.meia[1] * 2.0, pixels_per_meter);
        let casas = usize::from(unidade == DisplayUnit::Meters);
        let rotulo = ph2d_i18n::tr_with(
            "game.camera_frame.label",
            &[
                ("NAME", &m.nome),
                ("W", &format!("{w:.casas$}")),
                ("H", &format!("{h:.casas$}")),
                ("UNIT", &unidade.suffix()),
            ],
        );
        let tam = TypeToken::Sm.px();
        ph2d_editor_core::paint_text(
            texto,
            cena,
            &rotulo,
            rect.x0 as f32,
            rect.y0 as f32 - tam - ROTULO_FOLGA_PX,
            tam,
            ROTULO_MAX_PX,
            cor,
        );
        if !forte {
            continue;
        }
        for c in cantos(lo, hi) {
            alca(cena, tela(c), fundo, selecao);
        }
        if let Some((min, max)) = m.limites {
            let r = retangulo(tela([min[0], max[1]]), tela([max[0], min[1]]));
            cena.inner_mut().stroke(
                &Stroke::new(TRACO_PX[1]),
                Affine::IDENTITY,
                limites,
                None,
                &r,
            );
            for c in cantos(min, max) {
                alca(cena, tela(c), fundo, limites);
            }
            for c in [
                [min[0], (min[1] + max[1]) * 0.5],
                [max[0], (min[1] + max[1]) * 0.5],
                [(min[0] + max[0]) * 0.5, min[1]],
                [(min[0] + max[0]) * 0.5, max[1]],
                [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5],
            ] {
                alca(cena, tela(c), limites, fundo);
            }
        }
    }
}

fn retangulo(a: (f64, f64), b: (f64, f64)) -> ph2d_vector::Rect {
    ph2d_vector::Rect::new(a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1))
}

fn alca(cena: &mut VectorScene, c: (f64, f64), dentro: Color, borda: Color) {
    let r = ph2d_vector::Rect::new(
        c.0 - ALCA_PX,
        c.1 - ALCA_PX,
        c.0 + ALCA_PX,
        c.1 + ALCA_PX,
    );
    let s = cena.inner_mut();
    s.fill(Fill::NonZero, Affine::IDENTITY, dentro, None, &r);
    s.stroke(&Stroke::new(TRACO_PX[0]), Affine::IDENTITY, borda, None, &r);
}

/// ⭐ **As tarjas do palco** (plano GEF 17 D-F): no jogo a correr, o que fica fora da caixa do jogo dentro do
/// `canvas` pinta-se opaco — o jogador vê o formato do jogo, nunca mais mundo do que a moldura prometeu.
pub fn tarjas(cena: &mut VectorScene, canvas: [f32; 4], caixa: [f32; 4], theme: Theme) {
    let cor = ph2d_editor_core::paint::resolve(ColorToken::BgScrim, theme).with_alpha(1.0);
    let [x, y, w, h] = canvas.map(f64::from);
    let [cx, cy, cw, ch] = caixa.map(f64::from);
    let s = cena.inner_mut();
    for r in [
        ph2d_vector::Rect::new(x, y, x + w, cy),
        ph2d_vector::Rect::new(x, cy + ch, x + w, y + h),
        ph2d_vector::Rect::new(x, cy, cx, cy + ch),
        ph2d_vector::Rect::new(cx + cw, cy, x + w, cy + ch),
    ] {
        if r.width() > 0.0 && r.height() > 0.0 {
            s.fill(Fill::NonZero, Affine::IDENTITY, cor, None, &r);
        }
    }
}
