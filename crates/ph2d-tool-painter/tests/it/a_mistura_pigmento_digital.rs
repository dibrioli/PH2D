//! O Color mixing «Wet blend» do Digital com **Pigment 1** (azul + amarelo = verde; 08/11, a lei medida do CSP com a
//! cor do pigmento). Doc: `docs/Painter/CSP_COMO_MISTURA_2026-11-08.md`.
//! ⛔ O Watercolor Basic não muda: o hash do traço dele foi gravado na árvore ANTES da lei nova.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};

const W: usize = 400;
const H: usize = 240;
const Y: f32 = 120.0;
pub(crate) const AZUL: [u8; 3] = [0, 0, 220];
pub(crate) const VERMELHO: [u8; 3] = [200, 0, 0];
pub(crate) const AMARELO: [u8; 3] = [245, 205, 0];

pub(crate) fn arrasta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    let n = 60;
    for k in 1..=n {
        let u = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    t.on_tick(8.0);
}

pub(crate) fn mancha(t: &mut PainterTool, cor: [u8; 3]) {
    t.set_brush_color_srgb8(cor);
    for _ in 0..2 {
        for k in 0..9 {
            let y = Y - 40.0 + 10.0 * k as f32;
            arrasta(t, [140.0, y], [300.0, y]);
        }
    }
}

pub(crate) fn novo(preset: &str) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    pinceis(preset).remove(0).aplica(&mut t, true);
    t.set_brush_size_px(40.0);
    t
}

/// Watercolor Basic: mancha de `cor`, arrasto azul por cima até ao branco, com o Pigment `pig`.
pub(crate) fn basic(cor: [u8; 3], pig: f32) -> Vec<u8> {
    let mut t = novo("Watercolor Basic");
    t.set_brush_pigment_mixing(pig);
    mancha(&mut t, cor);
    t.set_brush_color_srgb8(AZUL);
    arrasta(&mut t, [60.0, Y], [340.0, Y]);
    t.run_full().0
}

fn fnv(p: &[u8]) -> u64 {
    p.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// ⛔ O Watercolor Basic dá o MESMO byte que antes da lei nova (azul sobre vermelho e sobre amarelo, Pigment 0 e 1).
/// Regravado na integração 13/11 pela lei B22.1 (secagem e encharque em passo fixo de 1/60 s, `a3ff312386` e
/// `c37c472dd9`; dono 10/11, «ritmo fixo de secagem»): com `PH2D_RITMO_FIXO=0` no mesmo binário voltam os de antes
/// (`0xae3e…4b35`, `0xbf2d…54e7`, `0xf37f…cc66`, `0xaeee…546a`).
#[test]
fn o_watercolor_basic_nao_mudou_com_o_pigment_mix() {
    let gravados: [(&str, [u8; 3], f32, u64); 4] = [
        ("vermelho pig 0", VERMELHO, 0.0, 0xbb5a_2a1b_7fd2_1c9f),
        ("vermelho pig 1", VERMELHO, 1.0, 0x1298_f3c3_1df1_cd27),
        ("amarelo pig 0", AMARELO, 0.0, 0x09ac_973c_f75a_3334),
        ("amarelo pig 1", AMARELO, 1.0, 0xd486_2c7f_b790_bd66),
    ];
    let lidos: Vec<(&str, u64, u64)> = gravados
        .iter()
        .map(|&(rot, cor, pig, h)| (rot, fnv(&basic(cor, pig)), h))
        .collect();
    for &(rot, l, h) in &lidos {
        assert_eq!(l, h, "Watercolor Basic {rot} mudou; lidos: {lidos:x?}");
    }
}

/// Mancha opaca (Color mixing Off) de `cor` com `preset`, depois o arrasto azul em Wet blend, Pigment 1, com a Density
/// dada.
pub(crate) fn digital(preset: &str, cor: Option<[u8; 3]>, densidade: f32) -> Vec<u8> {
    let mut t = novo(preset);
    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    if let Some(c) = cor {
        mancha(&mut t, c);
    }
    t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
    t.set_brush_mistura_numero(1, densidade);
    // Um Pigment só (`ac576c5df8`, handoff Painter 10/11 §4 «Cartão Paint Mixing»): o reservatório do Wet blend lê o
    // `effective_pigment_mix` do pincel — o antigo campo 5 da mistura saiu.
    t.set_brush_pigment_mixing(1.0);
    // Amount 0,15: com a amostra do CSP (o traço vê o fundo e a sua tinta de há mais de um raio) o azul a 0,3 sobre o
    // amarelo dá azul-petróleo `30,70,114` (medido 08/11), não verde.
    t.set_brush_mistura_numero(0, 0.15);
    t.set_brush_color_srgb8(AZUL);
    arrasta(&mut t, [60.0, Y], [340.0, Y]);
    t.run_full().0
}

fn px(p: &[u8], x: usize, y: usize) -> [i32; 4] {
    let i = (y * W + x) * 4;
    [p[i], p[i + 1], p[i + 2], p[i + 3]].map(i32::from)
}

/// ⭐ Azul sobre amarelo sai do azul em direção ao VERDE ao longo da mancha (G acima de R, B a descer), sem degrau
/// entre vizinhos em TODA a mancha (`x 160..290`, a costura da foto do dono 08/11); no G-Pen fica verde de `x 260`.
/// Azul sobre vermelho dá violeta/vinho (R e B acima de G). O limiar era `x 200` com o Pigment próprio do Wet blend;
/// com o Pigment ÚNICO (`ac576c5df8`, handoff Painter 10/11 §4 «Cartão Paint Mixing», §7) medido na integração 10/11:
/// G-Pen `x 200` `[93,130,148]` → `x 260` `[80,114,113]`; Hard Round `x 200` `[79,116,157]` → `x 290` `[71,104,106]`.
#[test]
fn o_pigment_mix_da_verde_continuo_e_violeta() {
    for preset in ["G-Pen", "Hard Round"] {
        let p = digital(preset, Some(AMARELO), 1.0);
        let mut ant: Option<[i32; 4]> = None;
        for x in 160..290 {
            let c = px(&p, x, Y as usize);
            assert!(
                c[1] > c[0] + 25,
                "{preset}: x {x} sem o verde do amarelo: {c:?}"
            );
            assert!(
                preset != "G-Pen" || x < 260 || (c[1] >= c[2] && c[1] > c[0] + 10),
                "{preset}: x {x} não é verde: {c:?}"
            );
            if let Some(a) = ant {
                let d = (0..3).map(|k| (c[k] - a[k]).abs()).max().unwrap_or(0);
                assert!(d <= 12, "{preset}: degrau {d} em x {x}: {a:?} → {c:?}");
                assert!(
                    c[2] <= a[2] + 2,
                    "{preset}: o azul volta a subir em x {x}: {a:?} → {c:?}"
                );
            }
            ant = Some(c);
        }
        let (ini, fim) = (px(&p, 160, Y as usize), px(&p, 289, Y as usize));
        assert!(
            fim[2] + 60 <= ini[2],
            "{preset}: o azul não desce ao longo do amarelo: {ini:?} → {fim:?}"
        );
        let v = digital(preset, Some(VERMELHO), 1.0);
        let c = px(&v, 220, Y as usize);
        // Pigment único: G-Pen `86,63,130` · Hard Round `75,58,140` (R − G era > 20; agora 17 no Hard Round).
        assert!(
            c[0] > c[1] + 10 && c[2] >= c[1] + 40,
            "{preset}: azul sobre vermelho não é vinho: {c:?}"
        );
    }
}

/// Camada vazia → a cor CHEIA do pincel no centro do traço (o vazio não é pigmento).
#[test]
fn o_pigment_mix_no_vazio_pinta_a_cor_do_pincel() {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    pinceis("G-Pen").remove(0).aplica(&mut t, true);
    t.set_brush_size_px(40.0);
    t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
    t.set_brush_color_srgb8(AZUL);
    arrasta(&mut t, [60.0, Y], [340.0, Y]);
    let p = t.run_full().0;
    let c = px(&p, 200, Y as usize);
    assert_eq!(c, [0, 0, 220, 255], "o vazio tem de levar a cor cheia");
    // O papel branco OPACO é cor (o CSP arrasta o branco): pouca tinta (Amount 0,15) sobre ele dá azul CLARO, nunca
    // cinzento nem transparente.
    let b = digital("G-Pen", None, 1.0);
    let c = px(&b, 200, Y as usize);
    assert!(
        c[3] == 255 && c[2] > c[0] + 100 && c[2] > c[1] + 80,
        "o branco opaco dá azul claro: {c:?}"
    );
}

/// Density baixa deixa ver o de baixo; o Sketch Pencil guarda o grão (desvio do ALFA na camada vazia ≥ 80 % do Off: a
/// cor da mistura sobre o branco é mais clara que a do Off, e o contraste da cor não mede o grão).
#[test]
fn o_pigment_mix_tem_see_through_e_guarda_o_grao() {
    // A Density é a força de cada carimbo (o CSP): só uma Density baixa deixa ver o amarelo.
    let cheio = digital("G-Pen", Some(AMARELO), 1.0);
    let meio = digital("G-Pen", Some(AMARELO), 0.05);
    let (a, b) = (px(&cheio, 220, Y as usize), px(&meio, 220, Y as usize));
    assert!(
        b[0] > a[0] + 20,
        "Density 0,05 tem de deixar ver o amarelo: {a:?} × {b:?}"
    );
    let desvio = |p: &[u8]| {
        let v: Vec<f32> = (100..300)
            .flat_map(|x| (Y as usize - 6..=Y as usize + 6).map(move |y| (x, y)))
            .map(|(x, y)| f32::from(p[(y * W + x) * 4 + 3]))
            .collect();
        let m = v.iter().sum::<f32>() / v.len() as f32;
        (v.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / v.len() as f32).sqrt()
    };
    let no_vazio = |modo: ModoMisturaOpaca| {
        let mut t = novo("Sketch Pencil");
        t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
        t.set_brush_mistura_modo(modo);
        t.set_brush_color_srgb8(AZUL);
        arrasta(&mut t, [60.0, Y], [340.0, Y]);
        t.run_full().0
    };
    let (off, mix) = (
        no_vazio(ModoMisturaOpaca::Off),
        no_vazio(ModoMisturaOpaca::WetBlend),
    );
    let (so, sm) = (desvio(&off), desvio(&mix));
    assert!(
        sm >= 0.8 * so,
        "o grão do lápis sumiu: desvio {sm} × Off {so}"
    );
}

/// O mesmo traço dá o mesmo byte (duas corridas no mesmo processo).
#[test]
fn o_pigment_mix_e_determinista() {
    assert_eq!(
        fnv(&digital("G-Pen", Some(AMARELO), 0.6)),
        fnv(&digital("G-Pen", Some(AMARELO), 0.6))
    );
}

/// Custo (à mão, `--profile smoke`): ms do arrasto azul (60 moves) por cima da mancha amarela — G-Pen Off, G-Pen
/// Pigment mix e Watercolor Basic (Pigment 1), intercalados no MESMO processo, o MÍNIMO de N blocos
/// (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medida: corre à mão"]
fn custo_do_pigment_mix() {
    let prepara = |v: usize| {
        let mut t = novo(if v == 2 { "Watercolor Basic" } else { "G-Pen" });
        if v == 2 {
            t.set_brush_pigment_mixing(1.0);
        } else {
            t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
        }
        mancha(&mut t, AMARELO);
        if v == 1 {
            t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
        }
        t.set_brush_color_srgb8(AZUL);
        t
    };
    let nomes = ["G-Pen Off", "G-Pen Pigment mix", "Watercolor Basic"];
    let mut min = [f64::MAX; 3];
    for bloco in 0..9 {
        for k in 0..3 {
            let v = (k + bloco) % 3;
            let mut t = prepara(v);
            let t0 = std::time::Instant::now();
            arrasta(&mut t, [60.0, Y], [340.0, Y]);
            let _ = std::hint::black_box(t.run_full());
            min[v] = min[v].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    for (n, m) in nomes.iter().zip(min) {
        eprintln!("[custo] {n}: {m:.2} ms (mínimo de 9)");
    }
}

/// A foto (à mão): `PH2D_MISTURA_FOTO=<ficheiro.ppm>` — colunas azul sobre amarelo · azul sobre vermelho; linhas
/// Watercolor Basic (Pigment 1), G-Pen, Hard Round e Sketch Pencil a Density 1 e 0,6; a última, azul sobre amarelo
/// numa camada VAZIA (G-Pen, Density 1) composta sobre o papel. Recorte `x 40..360 · y 60..180` de cada traço.
#[test]
#[ignore = "foto: corre à mão"]
fn foto_pigment_mix_digital() {
    let destino = std::env::var("PH2D_MISTURA_FOTO").expect("PH2D_MISTURA_FOTO");
    let (cw, ch) = (320usize, 120usize);
    let recorte = |p: &[u8]| -> Vec<[u8; 3]> {
        let mut v = Vec::with_capacity(cw * ch);
        for y in 60..60 + ch {
            for x in 40..40 + cw {
                let i = (y * W + x) * 4;
                // Sobre o papel branco (a camada vazia é transparente).
                let a = u32::from(p[i + 3]);
                v.push(
                    [0, 1, 2]
                        .map(|c| ((u32::from(p[i + c]) * a + 255 * (255 - a) + 127) / 255) as u8),
                );
            }
        }
        v
    };
    let mut linhas: Vec<[Vec<[u8; 3]>; 2]> = vec![[
        recorte(&basic(AMARELO, 1.0)),
        recorte(&basic(VERMELHO, 1.0)),
    ]];
    for preset in ["G-Pen", "Hard Round", "Sketch Pencil"] {
        for d in [1.0, 0.6] {
            linhas.push([
                recorte(&digital(preset, Some(AMARELO), d)),
                recorte(&digital(preset, Some(VERMELHO), d)),
            ]);
        }
    }
    let vazio = {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
        pinceis("G-Pen").remove(0).aplica(&mut t, true);
        t.set_brush_size_px(40.0);
        t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
        mancha(&mut t, AMARELO);
        t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
        t.set_brush_color_srgb8(AZUL);
        arrasta(&mut t, [60.0, Y], [340.0, Y]);
        recorte(&t.run_full().0)
    };
    linhas.push([vazio, vec![[128, 128, 128]; cw * ch]]);
    let gap = 6usize;
    let (fw, fh) = (2 * cw + 3 * gap, linhas.len() * (ch + gap) + gap);
    let mut img = vec![[60u8, 60, 60]; fw * fh];
    for (r, par) in linhas.iter().enumerate() {
        for (c, tile) in par.iter().enumerate() {
            let (ox, oy) = (gap + c * (cw + gap), gap + r * (ch + gap));
            for y in 0..ch {
                for x in 0..cw {
                    img[(oy + y) * fw + ox + x] = tile[y * cw + x];
                }
            }
        }
    }
    let mut ppm = format!("P6\n{fw} {fh}\n255\n").into_bytes();
    for p in img {
        ppm.extend_from_slice(&p);
    }
    std::fs::write(destino, ppm).expect("escreve");
}
