//! **As cenas 28–36** (P13 §8): cada uma parte de uma cena já montada (`base`), mexe no documento
//! pela lei do núcleo (`monta`) e pede o resto à família pelos mesmos pedidos do painel; `conduz`
//! abre a janela que a cena mostra.

use ph2d_editor_core::pixel_panel::{PixelLayerEffect, PixelRequest, PixelTool, PixelTweenEase};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_LINT, PANEL_PIXEL_PREVIEW};
use ph2d_pixel_core::audio::FaixaDeAudio;
use ph2d_pixel_core::cycle::{Cycle, CycleDirection};
use ph2d_pixel_core::dither::{Dither, PincelDither};
use ph2d_pixel_core::guia::Guias;
use ph2d_pixel_core::palette::DEFAULT as DB32;
use ph2d_pixel_core::{DocOp, NewSprite, Op, Patch, Sprite, apply};

/// O que a cena pede depois de montar.
pub type Pedidos = Vec<PixelRequest>;

/// Uma cena: a cena de base que a família encena primeiro e o que esta lhe acrescenta.
pub struct Cena {
    pub base: u32,
    pub monta: fn(&mut Sprite) -> Option<Pedidos>,
    /// A cena abre o objecto em Draw (a 34: a Preview só mostra um Pixel Sprite em Draw).
    pub draw: bool,
}

/// A cena `n` (28..=36).
#[must_use]
pub fn cena(n: u32) -> Option<Cena> {
    let (base, monta): (u32, fn(&mut Sprite) -> Option<Pedidos>) = match n {
        28 => (3, rampas),
        29 => (4, foto),
        30 => (4, dither),
        31 => (26, luz),
        32 => (4, guias),
        33 => (4, lint),
        34 => (26, |_| Some(Vec::new())),
        35 => (4, contorno),
        36 => (4, fala),
        _ => return None,
    };
    Some(Cena {
        base,
        monta,
        draw: n == 34,
    })
}

fn op(s: &mut Sprite, d: DocOp) -> Option<()> {
    apply(s, Op::Doc(Box::new(d))).ok().map(|_| ())
}

/// Um sprite RGB `w × h` com o RGBA dado no quadro e na camada de nascença.
fn tela(w: u32, h: u32, px: Vec<u8>) -> Option<Sprite> {
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        ..NewSprite::default()
    })
    .ok()?;
    let r = s.bounds();
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: px,
    };
    apply(&mut s, Op::Patch(p)).ok()?;
    Some(s)
}

/// As três rampas da cena 28 (índices da DB32): pele 5, água 4 (com ciclo), folha 6.
pub const RAMPAS_28: [(&str, &[u8]); 3] = [
    ("Skin", &[3, 4, 5, 6, 7]),
    ("Water", &[16, 17, 18, 19]),
    ("Leaf", &[9, 10, 11, 12, 13, 14]),
];
/// A rolagem do painel PIXEL na cena 28 (px lógicos): o cartão Palette e a faixa das rampas à vista.
pub const ROLAGEM_28: f32 = 700.0;
/// Os quadros em que a cena ainda arruma a vista (rolagem, tamanho da janela).
pub const QUADROS_DA_CENA: u32 = 120;
/// Quanto a janela Preview da 34 cresce (px lógicos: largura, altura).
pub const ESTICA_34: [f32; 2] = [60.0, 300.0];
/// O período do ciclo da água.
pub const CICLO_28_MS: u32 = 400;

fn rampas(s: &mut Sprite) -> Option<Pedidos> {
    for (name, ix) in RAMPAS_28 {
        op(
            s,
            DocOp::AddRamp {
                name: name.into(),
                indices: ix.to_vec(),
            },
        )?;
    }
    let ramp = s.ramps().ramps.get(1)?.id;
    let cycle = Cycle {
        ramp,
        periodo_ms: CICLO_28_MS,
        direcao: CycleDirection::Forward,
    };
    op(
        s,
        DocOp::SetCycle {
            ramp,
            cycle: Some(cycle),
        },
    )?;
    Some(vec![PixelRequest::FrameAction(
        ph2d_editor_core::pixel_panel::PixelFrameAction::Play,
    )])
}

/// Os três níveis do dither da cena 30 (de 64).
pub const NIVEIS_30: [u8; 3] = [16, 32, 48];

/// Pinta por cima do quadro 0 (a camada inteira num patch só: um patch PEQUENO encolheria o cel).
fn pinta(s: &mut Sprite, f: impl FnOnce(&mut dyn FnMut(i32, i32, [u8; 4]))) -> Option<()> {
    let (w, h) = (s.width() as i32, s.height() as i32);
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let mut px = Vec::new();
    ph2d_pixel_core::compose_rgba(s, frame, None, &mut px);
    f(&mut |x, y, c| {
        if (0..w).contains(&x) && (0..h).contains(&y) {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&c);
        }
    });
    let r = s.bounds();
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: px,
    };
    apply(s, Op::Patch(p)).ok().map(|_| ())
}

fn dither(s: &mut Sprite) -> Option<Pedidos> {
    let (w, faixa) = (s.width() as i32, s.height() as i32 / 4);
    pinta(s, |p| {
        for (i, nivel) in NIVEIS_30.iter().enumerate() {
            let d = PincelDither {
                nivel: *nivel,
                matriz: Dither::Bayer4,
            };
            let y0 = faixa / 2 + i as i32 * faixa + 2;
            for y in y0..y0 + faixa - 4 {
                for x in 4..w - 4 {
                    p(x, y, if d.primaria(x, y) { DB32[1] } else { DB32[22] });
                }
            }
        }
    })?;
    Some(vec![PixelRequest::Tool(PixelTool::Push)])
}

/// A cor do meio da rampa Skin da cena 31 (a rampa nasce dela, 5 degraus).
pub const PELE_31: [u8; 4] = [196, 120, 84, 255];
/// O lado da bola da cena 31.
pub const BOLA_31: u32 = 16;

/// A rampa Skin da cena 31, do escuro para o claro (o degrau do meio é [`PELE_31`]).
#[must_use]
pub fn rampa_31() -> Vec<[u8; 4]> {
    let cfg = ph2d_pixel_ponte::rampas::config(ph2d_editor_core::pixel_panel::PixelRampConfig {
        steps: 5,
        ..Default::default()
    });
    let mut r = ph2d_pixel_rampa::gera_rampa(PELE_31, cfg);
    r[2] = PELE_31;
    r
}

/// O herói da 31: uma bola 16 × 16 pintada TODA no degrau do meio da rampa Skin, com «Light:
/// Palette» ligado — a lâmpada da 26 à esquerda escolhe o degrau de cada lado.
fn luz(s: &mut Sprite) -> Option<Pedidos> {
    let pal = rampa_31();
    let n = BOLA_31 as i32;
    let px = (0..n * n)
        .flat_map(|k| {
            let (x, y) = (k % n - n / 2, k / n - n / 2);
            if x * x + y * y <= (n / 2 - 1).pow(2) {
                PELE_31
            } else {
                [0; 4]
            }
        })
        .collect();
    let spec = NewSprite {
        width: BOLA_31,
        height: BOLA_31,
        ..NewSprite::default()
    };
    let mut b = Sprite::with_palette(spec, &pal).ok()?;
    let r = b.bounds();
    let (layer, frame) = (b.layers()[0].id, b.frames()[0].id);
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: px,
    };
    apply(&mut b, Op::Patch(p)).ok()?;
    let add = DocOp::AddRamp {
        name: "Skin".into(),
        indices: (0..5).collect(),
    };
    op(&mut b, add)?;
    let o = ph2d_pixel_core::ramp::set_luz_na_paleta(&b, true)?;
    apply(&mut b, o).ok()?;
    *s = b;
    Some(Vec::new())
}

/// Os pontos de fuga e a grade iso da cena 32 (células).
pub const FUGA_32: [[f32; 2]; 2] = [[-8.0, 12.0], [72.0, 12.0]];
/// O fundo liso da cena 32 (um cinza claro da DB32).
pub const FUNDO_32: [u8; 4] = DB32[21];

fn guias(s: &mut Sprite) -> Option<Pedidos> {
    // Fundo liso (o gradiente da base competia com os raios) e só a perspectiva: o bloco iso
    // desenha-se à mão em 2:1, longe da linha guiada.
    *s = tela(64, 48, (0..64 * 48).flat_map(|_| FUNDO_32).collect())?;
    let g = Guias {
        pontos_de_fuga: [Some(FUGA_32[0]), Some(FUGA_32[1]), None],
        horizonte: Some(FUGA_32[0][1]),
        iso: None,
    };
    op(s, DocOp::SetGuias(g))?;
    // Uma linha guiada: do canto (20, 40) para o ponto de fuga da esquerda; e o bloco iso.
    pinta(s, |p| {
        let (a, f) = ([20.0f32, 40.0], FUGA_32[0]);
        for k in 0..=26 {
            let t = k as f32 / 40.0;
            let (x, y) = (a[0] + (f[0] - a[0]) * t, a[1] + (f[1] - a[1]) * t);
            p(x.round() as i32, y.round() as i32, DB32[27]);
        }
        // O bloco iso: o losango de cima (2:1) e as duas faces.
        for i in 0..8 {
            for (x, y) in [
                (44 + i, 26 - i / 2),
                (44 + i, 26 + i / 2),
                (52 + i, 22 + i / 2),
                (52 + i, 30 - i / 2),
                (44, 26 + i),
                (52, 30 + i),
                (59, 26 + i),
            ] {
                p(x, y, DB32[1]);
            }
        }
    })?;
    Some(Vec::new())
}

/// `vicios_p13` (a fixture do linter, `ph2d-pixel-lint/tests/it/fixture.rs`): um caso de cada vício.
#[must_use]
pub fn vicios_p13() -> Option<Sprite> {
    let mut px = vec![0u8; 16 * 16 * 4];
    let mut p = |x: u32, y: u32, c: [u8; 4]| {
        let i = ((y * 16 + x) * 4) as usize;
        px[i..i + 4].copy_from_slice(&c);
    };
    p(1, 1, DB32[28]);
    for x in 4..=7 {
        p(x, 3, DB32[1]);
    }
    for x in 7..=10 {
        p(x, 4, DB32[1]);
    }
    for x in 0..=2 {
        p(x, 6, DB32[1]);
    }
    p(3, 7, DB32[1]);
    for x in 4..=6 {
        p(x, 8, DB32[1]);
    }
    for x in 1..=4 {
        p(x, 10, DB32[7]);
        p(x, 11, DB32[6]);
        p(x, 12, DB32[4]);
    }
    for y in 8..=15u32 {
        for x in 8..=15u32 {
            let d = 1 + (x - 8).min(15 - x).min(y - 8).min(15 - y);
            let c = match d {
                1 => DB32[2],
                2 | 3 => DB32[16],
                _ => DB32[18],
            };
            p(x, y, c);
        }
    }
    tela(16, 16, px)
}

fn lint(s: &mut Sprite) -> Option<Pedidos> {
    *s = vicios_p13()?;
    Some(Vec::new())
}

/// A fixture do R3 (`ph2d-pixel-rampa/tests/fixtures/imagemagick_indexed.json`, gerada pelo oráculo
/// `docs/PixelArt/ferramentas/oraculo_indexed_imagemagick.py`).
const FIXTURE_29: &str =
    include_str!("../../ph2d-pixel-rampa/tests/fixtures/imagemagick_indexed.json");
/// As três rampas da fixture (pele, água, folha), índices da `paleta` dela.
pub const RAMPAS_29: [(&str, std::ops::Range<u8>); 3] =
    [("pele", 0..5), ("agua", 5..9), ("folha", 9..15)];
/// O lado (px) da `foto_p13` e o vão entre as duas metades da cena.
pub const FOTO_29: u32 = 64;
pub const VAO_29: u32 = 8;

fn depois_de<'a>(chave: &str) -> Option<&'a str> {
    let i = FIXTURE_29.find(chave)? + chave.len();
    Some(&FIXTURE_29[i..])
}

/// A `foto_p13` (RGB, 64 × 64) e a paleta da fixture.
#[must_use]
pub fn foto_p13() -> Option<(Vec<u8>, Vec<[u8; 4]>)> {
    let hex = depois_de("\"foto_p13_rgb\":\"")?;
    let hex = &hex[..hex.find('"')?];
    let rgb: Vec<u8> = (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
        .collect::<Option<_>>()?;
    let pal = depois_de("\"paleta\":")?;
    let pal = &pal[..pal.find("]]")?];
    let n: Vec<u8> = pal
        .split(|c: char| !c.is_ascii_digit())
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().ok())
        .collect::<Option<_>>()?;
    let pal = n.chunks_exact(3).map(|c| [c[0], c[1], c[2], 255]).collect();
    Some((rgb, pal))
}

fn com_rampas(rgba: Vec<u8>, w: u32, pal: &[[u8; 4]]) -> Option<Sprite> {
    let spec = NewSprite {
        width: w,
        height: FOTO_29,
        ..NewSprite::default()
    };
    let mut s = Sprite::with_palette(spec, pal).ok()?;
    let r = s.bounds();
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: rgba,
    };
    apply(&mut s, Op::Patch(p)).ok()?;
    for (name, ix) in RAMPAS_29 {
        let indices = ix.collect();
        op(
            &mut s,
            DocOp::AddRamp {
                name: name.into(),
                indices,
            },
        )?;
    }
    Some(s)
}

/// A foto convertida a Indexed por `w` (Exact RGB ou Keep ramps), de volta em RGBA.
fn em_rgba(w: &ph2d_pixel_core::Whole) -> Option<Vec<u8>> {
    let cel = w.cels.values().next()?;
    let mut px = vec![0u8; (FOTO_29 * FOTO_29 * 4) as usize];
    let b = cel.bbox;
    for (k, i) in cel.data.iter().enumerate() {
        let (x, y) = (b.x + (k as i32 % b.w as i32), b.y + (k as i32 / b.w as i32));
        let o = ((y as u32 * FOTO_29 + x as u32) * 4) as usize;
        px[o..o + 4].copy_from_slice(w.palette.get(usize::from(*i))?);
    }
    Some(px)
}

/// As duas metades da cena 29: (Exact RGB, Keep ramps), cada uma 64 × 64 RGBA.
#[must_use]
pub fn foto_lado_a_lado() -> Option<(Vec<u8>, Vec<u8>, Vec<[u8; 4]>)> {
    let (rgb, pal) = foto_p13()?;
    let rgba: Vec<u8> = rgb
        .chunks_exact(3)
        .flat_map(|c| [c[0], c[1], c[2], 255])
        .collect();
    let s = com_rampas(rgba, FOTO_29, &pal)?;
    let opt = ph2d_pixel_core::convert::Convert::default();
    let exata = ph2d_pixel_core::convert::convert(&s, ph2d_pixel_core::ColorMode::Indexed, opt)?;
    let rampas = ph2d_pixel_rampa::converte_preservando_rampas(&s, opt)?;
    Some((em_rgba(&exata)?, em_rgba(&rampas)?, pal))
}

fn foto(s: &mut Sprite) -> Option<Pedidos> {
    let (a, b, pal) = foto_lado_a_lado()?;
    let w = 2 * FOTO_29 + VAO_29;
    let mut px = vec![0u8; (w * FOTO_29 * 4) as usize];
    for y in 0..FOTO_29 as usize {
        let (l, o) = (FOTO_29 as usize * 4, y * w as usize * 4);
        px[o..o + l].copy_from_slice(&a[y * l..(y + 1) * l]);
        let o = o + (FOTO_29 + VAO_29) as usize * 4;
        px[o..o + l].copy_from_slice(&b[y * l..(y + 1) * l]);
    }
    *s = com_rampas(px, w, &pal)?;
    Some(Vec::new())
}

/// A cor do contorno da cena 35.
pub const CONTORNO_35: [u8; 4] = [250, 200, 0, 255];

fn contorno(s: &mut Sprite) -> Option<Pedidos> {
    let mut px = vec![0u8; 32 * 32 * 4];
    for y in 0..32u32 {
        for x in 0..32u32 {
            let (dx, dy) = (x as i32 - 16, y as i32 - 16);
            if dx * dx + dy * dy <= 81 {
                let i = ((y * 32 + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&DB32[9]);
            }
        }
    }
    *s = tela(32, 32, px)?;
    let id = s.layers()[0].id.0;
    Some(vec![PixelRequest::LayerEffect(
        id,
        PixelLayerEffect::On(CONTORNO_35),
    )])
}

/// O som curto da cena 36, gerado no arranque na pasta temporária: 0,6 s de seno de 440 Hz a
/// 22 050 Hz, mono 16 bits.
#[must_use]
pub fn fala_36() -> std::path::PathBuf {
    std::env::temp_dir().join("ph2d-p13-fala.wav")
}
/// Os quadros que o Tween gera entre A e B.
pub const TWEEN_36: u32 = 3;

fn wav() -> Option<std::path::PathBuf> {
    let p = fala_36();
    let (sr, n) = (22_050u32, 13_230u32);
    let mut b = Vec::with_capacity(44 + 2 * n as usize);
    let dados = 2 * n;
    for (k, v) in [(b"RIFF", 36 + dados), (b"WAVE", 0), (b"fmt ", 16)] {
        b.extend_from_slice(k);
        if k != b"WAVE" {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    for v in [1u16, 1] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&dados.to_le_bytes());
    for i in 0..n {
        let t = f64::from(i) / f64::from(sr);
        let v = ((t * 440.0 * std::f64::consts::TAU).sin() * 12_000.0) as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(&p, b).ok()?;
    Some(p)
}

/// A bola da cena 36: 9 × 9 px, num cel do tamanho dela (o Tween anda com a caixa do cel).
fn bola() -> Vec<u8> {
    (0..81)
        .flat_map(|k: i32| {
            let (x, y) = (k % 9 - 4, k / 9 - 4);
            if x * x + y * y <= 16 {
                DB32[27]
            } else {
                [0; 4]
            }
        })
        .collect()
}

/// A esquerda da bola em A e em B.
pub const BOLA_36: [i32; 2] = [1, 38];

fn fala(s: &mut Sprite) -> Option<Pedidos> {
    *s = tela(48, 16, vec![0; 48 * 16 * 4])?;
    let layer = s.layers()[0].id;
    let a = s.frames()[0].id;
    let (op_novo, b) =
        ph2d_pixel_core::frames::new_frame(s, a, ph2d_pixel_core::frames::NewFrame::Empty)?;
    apply(s, op_novo).ok()?;
    for (frame, x) in [(a, BOLA_36[0]), (b, BOLA_36[1])] {
        let r = ph2d_pixel_core::Rect::new(x, 4, 9, 9);
        let p = Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: bola(),
        };
        apply(s, Op::Patch(p)).ok()?;
    }
    let t =
        ph2d_pixel_ponte::quadros::tween(s, layer, &[a, b], a, TWEEN_36, PixelTweenEase::InOut, 0)?;
    apply(s, t).ok()?;
    let asset = wav()?.canonicalize().ok()?.to_string_lossy().into_owned();
    let faixa = FaixaDeAudio {
        asset,
        inicio_ms: 0,
        ganho: 1.0,
        mudo: false,
    };
    op(s, DocOp::SetAudio(vec![faixa]))?;
    Some(Vec::new())
}

/// Uma vez por quadro, com o nível armado: abre a janela que a cena mostra.
pub fn conduz(hero: &mut HeroScreen, nivel: Option<u32>) {
    // Os primeiros quadros (o painel nasce rolado a 0 quando a cena entra em Draw): depois, a roda é do dono.
    static Q: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let cedo = Q.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < QUADROS_DA_CENA;
    let painel = match nivel {
        // A 28 abre o painel PIXEL rolado até ao cartão Palette e à faixa das rampas.
        Some(28) if cedo => {
            let id = ph2d_editor_core::ids::PIXEL_PANEL;
            hero.store.set_panel_scroll(id, ROLAGEM_28);
            return;
        }
        Some(33) => PANEL_PIXEL_LINT,
        Some(34) => {
            let v = ph2d_pixel_crt::vista::vista();
            if !v.ligada {
                ph2d_pixel_crt::vista::define(ph2d_pixel_crt::vista::VistaCrt {
                    ligada: true,
                    ..v
                });
            }
            // A janela Preview esticada para baixo, como o dono a arrastaria pelo canto: o cartão
            // CRT come a altura de fábrica e a imagem ficava sem lugar.
            if cedo {
                let id = ph2d_editor_core::ids::PIXEL_PREVIEW_PANEL;
                hero.store
                    .set_panel_resize_delta(id, ESTICA_34[0], ESTICA_34[1]);
            }
            PANEL_PIXEL_PREVIEW
        }
        _ => return,
    };
    if !hero.is_panel_visible(painel) {
        hero.panel_visibility.insert(painel, true);
    }
}
