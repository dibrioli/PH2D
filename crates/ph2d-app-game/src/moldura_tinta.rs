//! A tinta da câmera no canvas (plano GEF 17 A1/A2/A4, cura 17b): o ícone (com o tamanho do projecto), a moldura
//! (forte com a câmera selecionada, ténue sem ela, tracejada se não é a que manda), o rótulo do tamanho DENTRO do
//! canvas, os cantos, os limites, a alça sob o ponteiro e — com `follow` — o fio até ao alvo e a mira.
//! Em espaço de TELA: o traço e o texto ficam nítidos em qualquer zoom.

use crate::moldura::{AGARRE_PX, Moldura, alca_em, caixa, cantos};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{CameraFrameShow, icons::IconId};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::{Affine, Color, Fill, Stroke, VectorScene};

/// A opacidade da moldura de uma câmera não selecionada (ténue, D-V «Sempre») e com o jogo a correr.
const TENUE: f32 = 0.45; // LITERAL-PX-OK: opacidade de overlay
const TRACO_PX: [f64; 2] = [1.0, 2.0]; // LITERAL-PX-OK: chrome de overlay (ténue, forte)
const TRACEJADO_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: chrome de overlay
const ALCA_PX: f64 = 4.0; // LITERAL-PX-OK: meia-alça
const ALCA_SOB_PX: f64 = 6.0; // LITERAL-PX-OK: meia-alça sob o ponteiro
const MIRA_PX: f64 = 5.0; // LITERAL-PX-OK: meio braço da cruz da mira
const ROTULO_FOLGA_PX: f32 = 4.0; // LITERAL-PX-OK: entre o rótulo e a moldura
const ROTULO_MAX_PX: f32 = 420.0; // LITERAL-PX-OK: largura máxima do rótulo

/// As molduras que se veem: todas com «Sempre», só a selecionada com «Só selecionada».
pub fn visiveis(ms: &[Moldura], mostrar: CameraFrameShow) -> impl Iterator<Item = &Moldura> {
    ms.iter()
        .filter(move |m| mostrar == CameraFrameShow::Always || m.selecionada)
}

/// O que o canvas sabe deste quadro para pintar a câmera.
#[derive(Clone, Copy, Debug)]
pub struct Pintura {
    pub mostrar: CameraFrameShow,
    pub unidade: DisplayUnit,
    pub pixels_per_meter: f32,
    pub icone_px: f32,
    /// O editor parado no início: alças e rótulo. Senão a moldura é o jogo (ténue, sem alças).
    pub em_edicao: bool,
    /// O rectângulo do canvas visível `[x, y, w, h]` (px de janela) — o rótulo fica dentro dele.
    pub canvas: [f32; 4],
    /// O ponteiro (px da vista), para acender a alça sob ele.
    pub ponteiro: (f32, f32),
}

/// O rectângulo do ícone de uma câmera em `pose`, em px da vista.
fn icone(pose: [f32; 2], lado: f32, camera: &Camera2d, window: WindowSize) -> Rect {
    let (x, y) = camera.world_to_screen(pose, window);
    Rect::new(x - lado * 0.5, y - lado * 0.5, lado, lado)
}

/// ⭐ **O ícone pega** (17b A9): as câmeras cujo ícone contém `tela` — a MESMA conta da tinta, no centro do quadro.
#[must_use]
pub fn icones_em(
    ms: &[Moldura],
    tela: (f32, f32),
    lado: f32,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<u64> {
    ms.iter()
        .filter(|m| icone(m.centro, lado, camera, window).contains(tela.0, tela.1))
        .map(|m| m.camera)
        .collect()
}

/// Pinta a câmera no canvas.
pub fn desenhar(
    ms: &[Moldura],
    p: Pintura,
    theme: Theme,
    camera: &Camera2d,
    window: WindowSize,
    cena: &mut VectorScene,
    texto: &mut TextSystem,
) {
    let selecao = ph2d_editor_core::paint::resolve(ColorToken::Selection, theme);
    let limites = ph2d_editor_core::paint::resolve(ColorToken::Warn, theme);
    let fundo = ph2d_editor_core::paint::resolve(ColorToken::Bg0, theme);
    let tela = |q: [f32; 2]| {
        let (x, y) = camera.world_to_screen(q, window);
        (f64::from(x), f64::from(y))
    };
    for m in ms {
        let r = icone(m.centro, p.icone_px, camera, window);
        ph2d_editor_core::paint_icon(cena, IconId::Camera, r, selecao, TRACO_PX[1] as f32);
    }
    let sob = {
        let a = camera.screen_to_world(p.ponteiro, window);
        let b = camera.screen_to_world((p.ponteiro.0 + 1.0, p.ponteiro.1), window);
        (a, (b[0] - a[0]).hypot(b[1] - a[1]) * AGARRE_PX)
    };
    for m in visiveis(ms, p.mostrar) {
        let forte = m.selecionada && p.em_edicao;
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
        cena.inner_mut()
            .stroke(&traco, Affine::IDENTITY, cor, None, &rect);
        if !p.em_edicao {
            continue;
        }
        rotulo(m, &p, rect, cor, cena, texto);
        if !forte {
            continue;
        }
        if let Some(s) = m.segue.as_ref() {
            fio_do_follow(m, s, cor, &tela, cena, texto);
        }
        let acesa = alca_em(m, sob.0, sob.1);
        let realce = |c: [f32; 2]| alca_em(m, c, 0.0) == acesa && acesa.is_some();
        for c in cantos(lo, hi) {
            alca(cena, tela(c), fundo, selecao, realce(c));
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
                alca(cena, tela(c), fundo, limites, realce(c));
            }
            for c in [
                [min[0], (min[1] + max[1]) * 0.5],
                [max[0], (min[1] + max[1]) * 0.5],
                [(min[0] + max[0]) * 0.5, min[1]],
                [(min[0] + max[0]) * 0.5, max[1]],
                [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5],
            ] {
                alca(cena, tela(c), limites, fundo, realce(c));
            }
        }
    }
}

/// O rótulo «Nome · W × H unidade» por cima da moldura, empurrado para DENTRO do canvas (17b A10).
fn rotulo(
    m: &Moldura,
    p: &Pintura,
    rect: ph2d_vector::Rect,
    cor: Color,
    cena: &mut VectorScene,
    texto: &mut TextSystem,
) {
    let w = p.unidade.from_meters(m.meia[0] * 2.0, p.pixels_per_meter);
    let h = p.unidade.from_meters(m.meia[1] * 2.0, p.pixels_per_meter);
    let casas = usize::from(p.unidade == DisplayUnit::Meters);
    let rotulo = ph2d_i18n::tr_with(
        "game.camera_frame.label",
        &[
            ("NAME", &m.nome),
            ("W", &format!("{w:.casas$}")),
            ("H", &format!("{h:.casas$}")),
            ("UNIT", &p.unidade.suffix()),
        ],
    );
    let tam = TypeToken::Sm.px();
    let [cx, cy, cw, ch] = p.canvas;
    let largura = texto.prefix_width(&rotulo, tam).min(ROTULO_MAX_PX);
    let x = (rect.x0 as f32).clamp(
        cx + ROTULO_FOLGA_PX,
        (cx + cw - largura - ROTULO_FOLGA_PX).max(cx),
    ); // CLAMP-OK: dentro do canvas
    let em_cima = rect.y0 as f32 - tam - ROTULO_FOLGA_PX;
    let y = if em_cima >= cy {
        em_cima
    } else {
        (rect.y0 as f32 + ROTULO_FOLGA_PX).clamp(cy, (cy + ch - tam).max(cy)) // CLAMP-OK: dentro do canvas
    };
    ph2d_editor_core::paint_text(texto, cena, &rotulo, x, y, tam, ROTULO_MAX_PX, cor);
}

/// ⭐ **O fio do `follow`** (17b A2): tracejado do ícone ao alvo, com o nome dele, e a cruz da MIRA (alvo + offset
/// do `follow`) — o que o miolo arrasta. A moldura vai até onde a cerca deixa; a cruz vai com o ponteiro.
fn fio_do_follow(
    m: &Moldura,
    s: &crate::moldura::Segue,
    cor: Color,
    tela: &dyn Fn([f32; 2]) -> (f64, f64),
    cena: &mut VectorScene,
    texto: &mut TextSystem,
) {
    let Some(alvo) = s.alvo else {
        return;
    };
    let (a, b) = (tela(m.centro), tela(alvo));
    let linha = ph2d_vector::Line::new(a, b);
    let s_ = cena.inner_mut();
    s_.stroke(
        &Stroke::new(TRACO_PX[0]).with_dashes(0.0, TRACEJADO_PX),
        Affine::IDENTITY,
        cor,
        None,
        &linha,
    );
    let mira = tela([alvo[0] + s.offset[0], alvo[1] + s.offset[1]]);
    for (d0, d1) in [
        ((MIRA_PX, 0.0), (-MIRA_PX, 0.0)),
        ((0.0, MIRA_PX), (0.0, -MIRA_PX)),
    ] {
        let l = ph2d_vector::Line::new(
            (mira.0 + d0.0, mira.1 + d0.1),
            (mira.0 + d1.0, mira.1 + d1.1),
        );
        s_.stroke(&Stroke::new(TRACO_PX[1]), Affine::IDENTITY, cor, None, &l);
    }
    let rotulo = ph2d_i18n::tr_with("game.camera_frame.follows", &[("TARGET", &s.nome)]);
    let tam = TypeToken::Sm.px();
    ph2d_editor_core::paint_text(
        texto,
        cena,
        &rotulo,
        b.0 as f32 + ROTULO_FOLGA_PX,
        b.1 as f32 + ROTULO_FOLGA_PX,
        tam,
        ROTULO_MAX_PX,
        cor,
    );
}

fn retangulo(a: (f64, f64), b: (f64, f64)) -> ph2d_vector::Rect {
    ph2d_vector::Rect::new(a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1))
}

fn alca(cena: &mut VectorScene, c: (f64, f64), dentro: Color, borda: Color, acesa: bool) {
    let (meia, dentro) = if acesa {
        (ALCA_SOB_PX, borda)
    } else {
        (ALCA_PX, dentro)
    };
    let r = ph2d_vector::Rect::new(c.0 - meia, c.1 - meia, c.0 + meia, c.1 + meia);
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
