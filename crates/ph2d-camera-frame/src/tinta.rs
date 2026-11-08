//! A tinta da câmera no canvas (plano GEF 17 A1/A2/A4, cura 17b): o ícone (com o tamanho do projecto), a moldura
//! (forte com a câmera selecionada, ténue sem ela, tracejada se não é a que manda), o rótulo do tamanho DENTRO do
//! canvas, os cantos, os limites, a alça sob o ponteiro e — com `follow` — o fio até ao alvo e a mira.
//! Em espaço de TELA: o traço e o texto ficam nítidos em qualquer zoom.

use crate::{AGARRE_PX, Alca, Moldura, alca_em, caixa, caixa_da_zona, cantos, pegas_da_zona};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::readout;
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
const HALO_PX: f64 = 3.0; // LITERAL-PX-OK: anel da alça sob o ponteiro (o halo do Vector)
const L_PX: f64 = 9.0; // LITERAL-PX-OK: braço do L dos cantos dos limites
const L_TRACO_PX: f64 = 3.0; // LITERAL-PX-OK: traço do L
const CONTORNO_PX: f64 = 1.0; // LITERAL-PX-OK: o contorno escuro por baixo das peças laranja
const BARRA_PX: f64 = 8.0; // LITERAL-PX-OK: meia-barra das bordas dos limites
const LOSANGO: f64 = 1.5; // LITERAL-PX-OK: o losango do centro, em meias-alças
const MIRA_PX: f64 = 5.0; // LITERAL-PX-OK: meio braço da cruz da mira
const ROTULO_FOLGA_PX: f32 = 4.0; // LITERAL-PX-OK: entre o rótulo e a moldura
const ROTULO_MAX_PX: f32 = 420.0; // LITERAL-PX-OK: largura máxima do rótulo
/// A tinta da moldura FORA da zona morta (onda 18): leve, o mundo lê-se por baixo; a zona fica limpa (livre).
const ZONA_TINTA: f32 = 0.14; // LITERAL-PX-OK: opacidade de overlay
/// O escuro de fora da moldura (onda 18, «Outside: Dimmed»): o mundo ainda se lê, a moldura manda.
const FORA_TINTA: f32 = 0.55; // LITERAL-PX-OK: opacidade de overlay

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
    /// O relógio parado (em qualquer instante): alças e rótulo. Senão a moldura é o jogo (ténue, sem alças).
    pub em_edicao: bool,
    /// O rectângulo do canvas visível `[x, y, w, h]` (px de janela) — o rótulo fica dentro dele.
    pub canvas: [f32; 4],
    /// O ponteiro (px da vista), para acender a alça sob ele.
    pub ponteiro: (f32, f32),
    /// «Outside: Dimmed» (onda 18): escurecer o canvas fora da moldura da selecionada (ou da activa).
    pub escurecer: bool,
    /// A alça agarrada da câmera selecionada (o arrasto armado): acesa e com a ficha, mesmo fora do agarre.
    pub agarrada: Option<Alca>,
}

/// ⭐ GEF 20 C5 — o canvas ÚTIL: as réguas ocupam a borda de cima e a da esquerda (o modelo do Figma), e um rótulo
/// encostado ali ficava por baixo delas («nera · 1422 × 800 px»).
#[must_use]
pub fn canvas_util(canvas: [f32; 4], reguas: bool) -> [f32; 4] {
    let r = if reguas {
        ph2d_editor_core::ruler::RULER_PX
    } else {
        0.0
    };
    [
        canvas[0] + r,
        canvas[1] + r,
        (canvas[2] - r).max(0.0),
        (canvas[3] - r).max(0.0),
    ]
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
    if p.escurecer
        && let Some(m) = ms
            .iter()
            .find(|m| m.selecionada)
            .or_else(|| ms.iter().find(|m| m.ativa))
    {
        let (lo, hi) = caixa(m);
        let r = retangulo(tela([lo[0], hi[1]]), tela([hi[0], lo[1]]));
        let [x, y, w, h] = p.canvas;
        let fora =
            ph2d_editor_core::paint::resolve(ColorToken::BgScrim, theme).with_alpha(FORA_TINTA);
        let canvas = ph2d_vector::Rect::new(
            f64::from(x),
            f64::from(y),
            f64::from(x + w),
            f64::from(y + h),
        );
        moldura_vazada(cena, canvas, r, fora);
    }
    // ⭐ A câmera selecionada acende na cor dela, a da Hierarchy (dono 08/11).
    let destaque = ph2d_editor_core::paint::resolve(ColorToken::ObjCamera, theme);
    for m in ms {
        let r = icone(m.centro, p.icone_px, camera, window);
        let cor = if m.selecionada { destaque } else { selecao };
        ph2d_editor_core::paint_icon(cena, IconId::Camera, r, cor, TRACO_PX[1] as f32);
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
        let acesa = p.agarrada.or_else(|| alca_em(m, sob.0, sob.1));
        if let Some((zlo, zhi)) = caixa_da_zona(m) {
            let z = retangulo(tela([zlo[0], zhi[1]]), tela([zhi[0], zlo[1]]));
            moldura_vazada(cena, rect, z, selecao.multiply_alpha(ZONA_TINTA));
            cena.inner_mut().stroke(
                &Stroke::new(TRACO_PX[0]).with_dashes(0.0, TRACEJADO_PX),
                Affine::IDENTITY,
                cor,
                None,
                &z,
            );
            etiqueta_da_caixa(z, "game.camera_frame.dead_zone_tag", cena, texto, theme);
            for (i, c) in pegas_da_zona(m, sob.1).into_iter().enumerate() {
                if let Some(c) = c {
                    let a = Alca::ZonaMorta(i as u8);
                    face(cena, tela(c), Cara::Zona, selecao, fundo, acesa == Some(a));
                }
            }
        }
        if let Some(s) = m.segue.as_ref() {
            fio_do_follow(m, s, cor, &p, &tela, (&mut *cena, &mut *texto), theme);
        }
        for (i, c) in cantos(lo, hi).into_iter().enumerate() {
            let a = Alca::Canto(i as u8);
            face(cena, tela(c), Cara::Zoom, selecao, fundo, acesa == Some(a));
        }
        if let Some((min, max)) = m.limites {
            let r = retangulo(tela([min[0], max[1]]), tela([max[0], min[1]]));
            // Um contorno escuro por baixo: o laranja dos limites lê-se sobre um céu claro (foto GEF 20).
            for (largura, tinta) in [
                (TRACO_PX[1] + 2.0 * CONTORNO_PX, fundo),
                (TRACO_PX[1], limites),
            ] {
                cena.inner_mut()
                    .stroke(&Stroke::new(largura), Affine::IDENTITY, tinta, None, &r);
            }
            etiqueta_da_caixa(r, "game.camera_frame.limits_tag", cena, texto, theme);
            for (i, c) in cantos(min, max).into_iter().enumerate() {
                let a = Alca::LimiteCanto(i as u8);
                face(
                    cena,
                    tela(c),
                    Cara::LimiteCanto(i as u8),
                    limites,
                    fundo,
                    acesa == Some(a),
                );
            }
            let meio = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
            for (i, c) in [
                [min[0], meio[1]],
                [max[0], meio[1]],
                [meio[0], min[1]],
                [meio[0], max[1]],
            ]
            .into_iter()
            .enumerate()
            {
                let a = Alca::LimiteBorda(i as u8);
                face(
                    cena,
                    tela(c),
                    Cara::LimiteBorda(i < 2),
                    limites,
                    fundo,
                    acesa == Some(a),
                );
            }
            let centro = Cara::LimiteCentro;
            face(
                cena,
                tela(meio),
                centro,
                limites,
                fundo,
                acesa == Some(Alca::LimiteCentro),
            );
        }
        // O miolo é a moldura INTEIRA: a ficha dele só a meio do arrasto, senão segue o ponteiro por todo o lado.
        if let Some(a) = acesa.filter(|a| *a != Alca::Dentro || p.agarrada.is_some()) {
            let txt = crate::etiqueta(m, a, p.unidade, p.pixels_per_meter);
            ficha_no_ponteiro(&txt, p, cena, texto, theme);
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

/// ⭐ **O fio do `follow`** (17b A2): tracejado do ícone ao alvo e a cruz da MIRA (alvo + offset do `follow`) — o que
/// o miolo arrasta. O nome do alvo vai numa ficha no MEIO do fio, dentro do canvas (GEF 20 C5: ao pé do alvo ele
/// ficava por baixo do boneco).
fn fio_do_follow(
    m: &Moldura,
    s: &crate::Segue,
    cor: Color,
    p: &Pintura,
    tela: &dyn Fn([f32; 2]) -> (f64, f64),
    (cena, texto): (&mut VectorScene, &mut TextSystem),
    theme: Theme,
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
    let w = readout::chip_width(texto, &rotulo, 0.0);
    let meio = [((a.0 + b.0) * 0.5) as f32, ((a.1 + b.1) * 0.5) as f32];
    readout::paint_chip(texto, cena, &rotulo, dentro(meio, w, p.canvas), 0.0, theme);
}

/// O centro de uma ficha de largura `w` em `c`, empurrado para dentro do canvas `[x, y, w, h]`.
fn dentro(c: [f32; 2], w: f32, canvas: [f32; 4]) -> [f32; 2] {
    let [x, y, cw, ch] = canvas;
    let (hw, hh) = (w * 0.5, readout::CHIP_H_PX * 0.5);
    [
        c[0].max(x + hw).min((x + cw - hw).max(x + hw)),
        c[1].max(y + hh).min((y + ch - hh).max(y + hh)),
    ]
}

/// ⭐ GEF 20 C4 — a ficha do nome e do valor da alça junto do ponteiro (a mesma das alças das formas do Vector).
fn ficha_no_ponteiro(
    txt: &str,
    p: Pintura,
    cena: &mut VectorScene,
    texto: &mut TextSystem,
    theme: Theme,
) {
    let w = readout::chip_width(texto, txt, 0.0);
    let [x, y, cw, ch] = p.canvas;
    let at = readout::at_cursor([p.ponteiro.0, p.ponteiro.1], w, Rect::new(x, y, cw, ch));
    readout::paint_chip(texto, cena, txt, at, 0.0, theme);
}

/// ⭐ GEF 20 C4 — o NOME de uma caixa («Limits», «Dead Zone») numa ficha no canto de cima-esquerdo, por dentro;
/// só se cabe.
fn etiqueta_da_caixa(
    r: ph2d_vector::Rect,
    chave: &str,
    cena: &mut VectorScene,
    texto: &mut TextSystem,
    theme: Theme,
) {
    let nome = ph2d_i18n::tr(chave);
    let w = readout::chip_width(texto, nome, 0.0);
    let folga = f64::from(ROTULO_FOLGA_PX);
    let h = f64::from(readout::CHIP_H_PX);
    if r.width() < f64::from(w) + 2.0 * folga || r.height() < h + 2.0 * folga {
        return;
    }
    let centro = [
        (r.x0 + folga) as f32 + w * 0.5,
        (r.y0 + folga + h * 0.5) as f32,
    ];
    readout::paint_chip(texto, cena, nome, centro, 0.0, theme);
}

/// Pinta `fora` menos `dentro` (quatro faixas; `dentro` cortado a `fora`) — o escuro de fora e a tinta da zona morta.
fn moldura_vazada(
    cena: &mut VectorScene,
    fora: ph2d_vector::Rect,
    dentro: ph2d_vector::Rect,
    cor: Color,
) {
    let d = dentro.intersect(fora);
    let d = if d.width() > 0.0 && d.height() > 0.0 {
        d
    } else {
        ph2d_vector::Rect::new(fora.x0, fora.y0, fora.x0, fora.y0)
    };
    let s = cena.inner_mut();
    for r in [
        ph2d_vector::Rect::new(fora.x0, fora.y0, fora.x1, d.y0),
        ph2d_vector::Rect::new(fora.x0, d.y1, fora.x1, fora.y1),
        ph2d_vector::Rect::new(fora.x0, d.y0, d.x0, d.y1),
        ph2d_vector::Rect::new(d.x1, d.y0, fora.x1, d.y1),
    ] {
        if r.width() > 0.0 && r.height() > 0.0 {
            s.fill(Fill::NonZero, Affine::IDENTITY, cor, None, &r);
        }
    }
}

fn retangulo(a: (f64, f64), b: (f64, f64)) -> ph2d_vector::Rect {
    ph2d_vector::Rect::new(a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1))
}

/// ⭐ GEF 20 C4 — **a cara de cada família de alça** (o idioma das alças das formas do Vector, `shape_handles_face`):
/// o zoom é o quadrado dos cantos de uma caixa; a zona morta, círculos; os limites, cantos em L, barras ao longo
/// da borda e o losango de mover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cara {
    Zoom,
    Zona,
    /// O canto `i` dos limites (a ordem de [`crate::cantos`]): os braços do L vão para DENTRO da caixa.
    LimiteCanto(u8),
    /// Uma borda dos limites: `true` = vertical (esquerda/direita).
    LimiteBorda(bool),
    LimiteCentro,
}

fn face(
    cena: &mut VectorScene,
    c: (f64, f64),
    cara: Cara,
    tinta: Color,
    fundo: Color,
    acesa: bool,
) {
    let s = cena.inner_mut();
    let k = if acesa { ALCA_SOB_PX / ALCA_PX } else { 1.0 };
    let m = ALCA_PX * k;
    if acesa {
        let halo = ph2d_vector::Circle::new(c, m + HALO_PX);
        s.stroke(
            &Stroke::new(TRACO_PX[0]),
            Affine::IDENTITY,
            tinta.multiply_alpha(TENUE),
            None,
            &halo,
        );
    }
    match cara {
        Cara::Zoom => {
            let r = ph2d_vector::Rect::new(c.0 - m, c.1 - m, c.0 + m, c.1 + m);
            s.fill(Fill::NonZero, Affine::IDENTITY, fundo, None, &r);
            s.stroke(&Stroke::new(TRACO_PX[1]), Affine::IDENTITY, tinta, None, &r);
        }
        Cara::Zona => {
            let r = ph2d_vector::Circle::new(c, m);
            s.fill(Fill::NonZero, Affine::IDENTITY, tinta, None, &r);
            s.stroke(&Stroke::new(TRACO_PX[0]), Affine::IDENTITY, fundo, None, &r);
        }
        Cara::LimiteCanto(i) => {
            // Na tela o y desce: os cantos de baixo (0, 1) abrem para CIMA.
            let dx = if i == 0 || i == 3 { 1.0 } else { -1.0 };
            let dy = if i < 2 { -1.0 } else { 1.0 };
            let l = L_PX * k;
            let mut p = ph2d_vector::BezPath::new();
            p.move_to((c.0 + dx * l, c.1));
            p.line_to(c);
            p.line_to((c.0, c.1 + dy * l));
            for (largura, cor) in [(L_TRACO_PX + 2.0 * CONTORNO_PX, fundo), (L_TRACO_PX, tinta)] {
                s.stroke(&Stroke::new(largura), Affine::IDENTITY, cor, None, &p);
            }
        }
        Cara::LimiteBorda(vertical) => {
            let (hw, hh) = if vertical {
                (m * 0.5, BARRA_PX * k)
            } else {
                (BARRA_PX * k, m * 0.5)
            };
            let r = ph2d_vector::RoundedRect::new(c.0 - hw, c.1 - hh, c.0 + hw, c.1 + hh, m * 0.5);
            s.fill(Fill::NonZero, Affine::IDENTITY, tinta, None, &r);
            s.stroke(&Stroke::new(CONTORNO_PX), Affine::IDENTITY, fundo, None, &r);
        }
        Cara::LimiteCentro => {
            let mut p = ph2d_vector::BezPath::new();
            let d = m * LOSANGO;
            p.move_to((c.0, c.1 - d));
            p.line_to((c.0 + d, c.1));
            p.line_to((c.0, c.1 + d));
            p.line_to((c.0 - d, c.1));
            p.close_path();
            s.fill(Fill::NonZero, Affine::IDENTITY, fundo, None, &p);
            s.stroke(&Stroke::new(TRACO_PX[1]), Affine::IDENTITY, tinta, None, &p);
        }
    }
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

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
