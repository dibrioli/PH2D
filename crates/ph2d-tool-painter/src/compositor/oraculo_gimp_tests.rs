//! O compositor contra o ORÁCULO das camadas: o GIMP corrido sem interface sobre entradas nossas
//! (`docs/Painter/ferramentas/oraculo_camadas_gimp/`, doc Painter 45 §5). Cada corrida é uma base e
//! UMA camada por cima (rampa de alfa × cores × modo × opacidade), num espaço: «perceptual» (sRGB
//! codificado) ou «linear» (luz).

use super::*;
use crate::layers::LayerStack;

static FIXTURA: &[u8] =
    include_bytes!("../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/gimp_3.2.6.bin");

struct Corrida {
    modo: BlendMode,
    nome: &'static str,
    espaco: &'static str,
    opacidade: u32,
    px: &'static [u8],
}

struct Fixtura {
    w: u32,
    h: u32,
    base: &'static [u8],
    topo: &'static [u8],
    corridas: Vec<Corrida>,
}

fn linha(buf: &'static [u8], at: &mut usize) -> &'static str {
    let fim = *at
        + buf[*at..]
            .iter()
            .position(|&b| b == b'\n')
            .expect("linha sem fim");
    let s = std::str::from_utf8(&buf[*at..fim]).expect("cabeçalho não é texto");
    *at = fim + 1;
    s
}

fn modo_por_nome(nome: &str) -> BlendMode {
    (0..ph2d_painter_effects::MAX_BLEND_MODES)
        .map(BlendMode::from_u8)
        .find(|m| format!("{m:?}") == nome)
        .unwrap_or_else(|| panic!("modo desconhecido na fixtura: {nome}"))
}

fn le() -> Fixtura {
    let mut at = 0;
    let (mut w, mut h) = (0u32, 0u32);
    loop {
        let l = linha(FIXTURA, &mut at);
        if l == "#FIM" {
            break;
        }
        if let Some(r) = l.strip_prefix("# grelha ....: W=") {
            let mut it = r
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty());
            w = it.next().unwrap().parse().unwrap();
            h = it.next().unwrap().parse().unwrap();
        }
    }
    let n = (w * h * 4) as usize;
    let mut bloco = |nome: &str, at: &mut usize| {
        assert_eq!(linha(FIXTURA, at), nome);
        let b = &FIXTURA[*at..*at + n];
        *at += n;
        b
    };
    let base = bloco("BASE", &mut at);
    let topo = bloco("TOPO", &mut at);
    let mut corridas = Vec::new();
    while at < FIXTURA.len() {
        let l = linha(FIXTURA, &mut at);
        let mut c = l.split(' ');
        assert_eq!(c.next(), Some("RUN"));
        let nome = c.next().unwrap();
        let _gimp = c.next();
        let espaco = c.next().unwrap();
        let opacidade = c.next().unwrap().parse().unwrap();
        corridas.push(Corrida {
            modo: modo_por_nome(nome),
            nome,
            espaco,
            opacidade,
            px: &FIXTURA[at..at + n],
        });
        at += n;
    }
    Fixtura {
        w,
        h,
        base,
        topo,
        corridas,
    }
}

/// O compositor de produção sobre a entrada da fixtura: a base em `Normal` e a camada no modo e
/// opacidade da corrida.
fn compoe(f: &Fixtura, c: &Corrida) -> Vec<u8> {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", f.w, f.h).unwrap();
    let topo = s.add_raster("topo", f.w, f.h).unwrap();
    let l = s.get_mut(topo).unwrap();
    l.blend_mode = c.modo;
    l.opacity = c.opacidade as f32 / 100.0;
    let mut src = MapPixelSource::default();
    let img = |px: &[u8]| LayerImage {
        width: f.w,
        height: f.h,
        rgba8: px.to_vec(),
    };
    src.insert(base, img(f.base));
    src.insert(topo, img(f.topo));
    composite(&s, &src, f.w, f.h)
}

/// `(pior desvio em degraus, quantos canais a mais de 1)`. Onde o GIMP dá alfa 0 a cor é
/// indefinida: só o alfa conta.
fn desvio(nosso: &[u8], gimp: &[u8]) -> (u8, usize) {
    let mut pior = 0u8;
    let mut fora = 0;
    for (a, b) in nosso.as_chunks::<4>().0.iter().zip(gimp.as_chunks::<4>().0) {
        let canais = if b[3] == 0 { 3..4 } else { 0..4 };
        for k in canais {
            let d = a[k].abs_diff(b[k]);
            pior = pior.max(d);
            fora += usize::from(d > 1);
        }
    }
    (pior, fora)
}

/// Os modos cuja FÓRMULA o GIMP partilha com o W3C (medido a 03/10: o «linear» do GIMP reproduz o
/// compositor de então a ≤ 1 degrau). Os outros divergem nos DOIS espaços — o GIMP em vírgula
/// flutuante não corta `B(Cb, Cs)` a `[0, 1]` (Add, Burn, Dodge, Linear*), o Soft Light é outra
/// fórmula, os HSL são HSV/HSL/LCh e o Erase não é o Clear: é a P2 (doc Painter 45 §6).
const MODOS_DE_FORMULA_PARTILHADA: [BlendMode; 10] = [
    BlendMode::Normal,
    BlendMode::Multiply,
    BlendMode::Darken,
    BlendMode::Lighten,
    BlendMode::Screen,
    BlendMode::Overlay,
    BlendMode::HardLight,
    BlendMode::VividLight,
    BlendMode::Difference,
    BlendMode::Exclusion,
];

/// A matemática W3C de [`apply_blend`] sobre a entrada da fixtura, com a transferência dada
/// (`decode` byte → canal, `encode` canal → byte): a base sobre o vazio, a camada por cima.
fn mistura(
    f: &Fixtura,
    c: &Corrida,
    decode: impl Fn(u8) -> f32,
    encode: impl Fn(f32) -> u8,
) -> Vec<u8> {
    let px = |b: &[u8]| {
        [
            decode(b[0]),
            decode(b[1]),
            decode(b[2]),
            b[3] as f32 / 255.0,
        ]
    };
    let mut out = Vec::with_capacity(f.base.len());
    for (b, t) in f
        .base
        .as_chunks::<4>()
        .0
        .iter()
        .zip(f.topo.as_chunks::<4>().0)
    {
        let acc = apply_blend(BlendMode::Normal, [0.0; 4], px(b));
        let mut s = px(t);
        s[3] *= c.opacidade as f32 / 100.0;
        let o = apply_blend(c.modo, acc, s);
        out.extend([
            encode(o[0]),
            encode(o[1]),
            encode(o[2]),
            (o[3].clamp(0.0, 1.0) * 255.0).round() as u8,
        ]);
    }
    out
}

fn em_luz(f: &Fixtura, c: &Corrida) -> Vec<u8> {
    use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
    mistura(f, c, srgb_to_linear_byte, linear_to_srgb_byte)
}

fn em_tons_de_ecra(f: &Fixtura, c: &Corrida) -> Vec<u8> {
    mistura(
        f,
        c,
        |b| b as f32 / 255.0,
        |v| (v.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// ⭐ O CONTROLO da régua (doc Painter 45 §6, P0): o GIMP no espaço «linear» é a matemática W3C EM
/// LUZ, a um degrau, em todo modo de fórmula partilhada. Se isto cair, o oráculo deixou de medir o
/// que diz medir — e nenhuma comparação «perceptual» vale.
#[test]
fn o_gimp_linear_e_a_mistura_w3c_em_luz() {
    let f = le();
    let mut vistas = 0;
    for c in f.corridas.iter().filter(|c| c.espaco == "linear") {
        if !MODOS_DE_FORMULA_PARTILHADA.contains(&c.modo) {
            continue;
        }
        vistas += 1;
        let (pior, fora) = desvio(&em_luz(&f, c), c.px);
        assert!(
            pior <= 1,
            "{} {}%: pior {pior} degraus ({fora} canais)",
            c.nome,
            c.opacidade
        );
    }
    // CONTROLO do filtro: cada modo partilhado, nas duas opacidades.
    assert_eq!(vistas, 2 * MODOS_DE_FORMULA_PARTILHADA.len());
}

/// ⭐ A LEI (ADR-0177, P1 gate a): o compositor de produção é o GIMP «perceptual» a um degrau em
/// todo modo de fórmula partilhada, nas duas opacidades, com a base opaca e translúcida.
#[test]
fn o_compositor_junta_as_camadas_como_o_gimp_perceptual() {
    let f = le();
    let mut vistas = 0;
    for c in f.corridas.iter().filter(|c| c.espaco == "perceptual") {
        if !MODOS_DE_FORMULA_PARTILHADA.contains(&c.modo) {
            continue;
        }
        vistas += 1;
        let (pior, fora) = desvio(&compoe(&f, c), c.px);
        assert!(
            pior <= 1,
            "{} {}%: pior {pior} degraus ({fora} canais)",
            c.nome,
            c.opacidade
        );
    }
    assert_eq!(vistas, 2 * MODOS_DE_FORMULA_PARTILHADA.len());
}

/// Sonda: a tabela inteira — cada corrida do oráculo contra o compositor de produção, contra a
/// matemática em luz e contra a mesma matemática em tons de ecrã.
#[test]
#[ignore = "sonda: imprime a tabela do oráculo"]
fn diag_o_compositor_contra_o_gimp() {
    let f = le();
    println!("modo        espaço      op   | compositor  | em luz      | em ecrã");
    for c in &f.corridas {
        let col = |v: Vec<u8>| {
            let (p, n) = desvio(&v, c.px);
            format!("{p:>3} ({n:>4})")
        };
        println!(
            "{:<11} {:<10} {:>3}%  | {} | {} | {}",
            c.nome,
            c.espaco,
            c.opacidade,
            col(compoe(&f, c)),
            col(em_luz(&f, c)),
            col(em_tons_de_ecra(&f, c)),
        );
    }
}
