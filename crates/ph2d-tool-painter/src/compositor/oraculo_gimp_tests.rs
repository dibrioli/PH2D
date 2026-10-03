//! O compositor contra os ORÁCULOS das camadas: o GIMP e o Krita corridos sem interface sobre
//! entradas nossas (`docs/Painter/ferramentas/oraculo_camadas_gimp/`, doc Painter 45 §5). Cada
//! corrida é uma base e UMA camada por cima (rampa de alfa × cores × modo × opacidade), num espaço:
//! o GIMP em «perceptual» (sRGB codificado) ou «linear» (luz); o Krita a 8 bits (`krita-u8`,
//! codificado).

use super::*;
use crate::layers::LayerStack;

static GIMP: &[u8] =
    include_bytes!("../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/gimp_3.2.6.bin");
static KRITA: &[u8] =
    include_bytes!("../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/krita_6.0.4.bin");

struct Corrida {
    modo: BlendMode,
    nome: &'static str,
    /// O modo do oráculo que diz ser o nosso (o id do GIMP ou do Krita).
    deles: &'static str,
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
    le_de(GIMP)
}

fn le_de(fixtura: &'static [u8]) -> Fixtura {
    let mut at = 0;
    let (mut w, mut h) = (0u32, 0u32);
    loop {
        let l = linha(fixtura, &mut at);
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
    let bloco = |nome: &str, at: &mut usize| {
        assert_eq!(linha(fixtura, at), nome);
        let b = &fixtura[*at..*at + n];
        *at += n;
        b
    };
    let base = bloco("BASE", &mut at);
    let topo = bloco("TOPO", &mut at);
    let mut corridas = Vec::new();
    while at < fixtura.len() {
        let l = linha(fixtura, &mut at);
        let mut c = l.split(' ');
        assert_eq!(c.next(), Some("RUN"));
        let nome = c.next().unwrap();
        let deles = c.next().unwrap();
        let espaco = c.next().unwrap();
        let opacidade = c.next().unwrap().parse().unwrap();
        corridas.push(Corrida {
            modo: modo_por_nome(nome),
            nome,
            deles,
            espaco,
            opacidade,
            px: &fixtura[at..at + n],
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

/// O source-over do W3C com uma função de mistura separável DADA (sem o corte final de `B`), sobre
/// valores codificados: a régua para dizer QUE fórmula um oráculo usa num modo que diverge.
fn w3c_com(f: &Fixtura, c: &Corrida, b: fn(f32, f32) -> f32) -> Vec<u8> {
    let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = Vec::with_capacity(f.base.len());
    for (pb, pt) in f
        .base
        .as_chunks::<4>()
        .0
        .iter()
        .zip(f.topo.as_chunks::<4>().0)
    {
        let ab = pb[3] as f32 / 255.0;
        let as_ = pt[3] as f32 / 255.0 * c.opacidade as f32 / 100.0;
        let ao = as_ + ab * (1.0 - as_);
        for k in 0..3 {
            let (cb, cs) = (pb[k] as f32 / 255.0, pt[k] as f32 / 255.0);
            let co = if ao > 0.0 {
                (as_ * ((1.0 - ab) * cs + ab * b(cb, cs)) + ab * (1.0 - as_) * cb) / ao
            } else {
                0.0
            };
            out.push(enc(co));
        }
        out.push(enc(ao));
    }
    out
}

/// As fórmulas candidatas por modo (o nome diz de onde vêm). `B` SEM corte a `[0, 1]`.
/// Uma fórmula candidata: o nome (de onde vem) e a função de mistura separável.
type Candidata = (&'static str, fn(f32, f32) -> f32);

fn candidatas(m: BlendMode) -> Vec<Candidata> {
    match m {
        BlendMode::Add => vec![("cb+cs sem corte", |b, s| b + s)],
        BlendMode::LinearBurn => vec![("cb+cs-1 sem corte", |b, s| b + s - 1.0)],
        BlendMode::LinearLight => vec![("cb+2cs-1 sem corte", |b, s| b + 2.0 * s - 1.0)],
        BlendMode::ColorDodge => vec![("cb/(1-cs) sem corte", |b, s| {
            if s < 1.0 {
                b / (1.0 - s)
            } else if b > 0.0 {
                1e9
            } else {
                0.0
            }
        })],
        BlendMode::ColorBurn => vec![("1-(1-cb)/cs sem corte", |b, s| {
            if s > 0.0 {
                1.0 - (1.0 - b) / s
            } else if b < 1.0 {
                -1e9
            } else {
                1.0
            }
        })],
        BlendMode::SoftLight => vec![
            ("Photoshop", |b, s| {
                if s <= 0.5 {
                    2.0 * b * s + b * b * (1.0 - 2.0 * s)
                } else {
                    2.0 * b * (1.0 - s) + b.sqrt() * (2.0 * s - 1.0)
                }
            }),
            ("Pegtop", |b, s| (1.0 - 2.0 * s) * b * b + 2.0 * s * b),
        ],
        _ => vec![],
    }
}

/// Sonda da P2: cada corrida dos DOIS oráculos contra o compositor de produção e contra cada fórmula
/// candidata do modo — o pior desvio e, entre parênteses, os canais a mais de 1.
#[test]
#[ignore = "sonda: imprime a tabela dos dois oráculos por fórmula"]
fn diag_que_formula_cada_oraculo_usa() {
    for (nome, fx) in [("GIMP", GIMP), ("Krita", KRITA)] {
        let f = le_de(fx);
        println!("── {nome}");
        for c in f
            .corridas
            .iter()
            .filter(|c| c.espaco != "linear" && !MODOS_DE_FORMULA_PARTILHADA.contains(&c.modo))
        {
            let col = |v: &[u8]| {
                let (p, n) = desvio(v, c.px);
                format!("{p:>3} ({n:>4})")
            };
            let mut linha = format!(
                "{:<11} {:<16} {:>3}% | nosso {}",
                c.nome,
                c.deles,
                c.opacidade,
                col(&compoe(&f, c))
            );
            for (cn, b) in candidatas(c.modo) {
                linha += &format!(" | {cn} {}", col(&w3c_com(&f, c, b)));
            }
            println!("{linha}");
        }
        // E o Krita nos modos partilhados (a 2.ª opinião da lei).
        if nome == "Krita" {
            for c in f
                .corridas
                .iter()
                .filter(|c| MODOS_DE_FORMULA_PARTILHADA.contains(&c.modo))
            {
                let (p, n) = desvio(&compoe(&f, c), c.px);
                println!(
                    "{:<11} {:<16} {:>3}% | nosso {p:>3} ({n:>4})",
                    c.nome, c.deles, c.opacidade
                );
            }
        }
    }
}

/// ⭐ A 2.ª opinião da LEI (ADR-0177, P2): o Krita a 8 bits compõe no espaço codificado, e o nosso
/// compositor é ele em TODOS os modos que ele exprime — os 22 menos nenhum (Clear = o `erase`
/// dele). O resto é o arredondamento inteiro do Krita: medido a 03/10, pior `4` degraus e no máximo
/// `93` canais de `4 896` a mais de 1; o tecto é esse número com folga, nunca um modo inteiro fora.
#[test]
fn o_compositor_junta_as_camadas_como_o_krita_a_8_bits() {
    let f = le_de(KRITA);
    let mut modos = std::collections::BTreeSet::new();
    for c in &f.corridas {
        // Os ids do Krita com OUTRA fórmula que a nossa (o modelo HSL dele, a lightness).
        if matches!(
            c.deles,
            "hue_hsl" | "saturation_hsl" | "color_hsl" | "lightness"
        ) {
            continue;
        }
        let (pior, fora) = desvio(&compoe(&f, c), c.px);
        assert!(
            pior <= 4 && fora <= 120,
            "{} ({}) {}%: pior {pior} degraus, {fora} canais a mais de 1",
            c.nome,
            c.deles,
            c.opacidade
        );
        modos.insert(c.modo.to_u8());
    }
    // CONTROLO: os 22 modos passaram pela régua (nenhum ficou sem oráculo).
    assert_eq!(modos.len(), ph2d_painter_effects::MAX_BLEND_MODES as usize);
}

/// As diferenças do GIMP (P2), DOCUMENTADAS e não «aceites»: em cada modo onde ele diverge do
/// W3C, o «perceptual» dele é EXACTAMENTE a fórmula nomeada — `B` sem o corte a `[0, 1]` (a
/// vírgula flutuante dele) e o Soft Light Pegtop. O W3C, o Photoshop e o Krita cortam, e o Soft
/// Light deles é o do W3C/Photoshop: a nossa fica.
#[test]
fn as_divergencias_do_gimp_sao_as_formulas_nomeadas() {
    let f = le();
    let mut vistas = 0;
    for c in f.corridas.iter().filter(|c| c.espaco == "perceptual") {
        let quem = match c.modo {
            BlendMode::SoftLight => "Pegtop",
            _ => match candidatas(c.modo).first() {
                Some((n, _)) => n,
                None => continue,
            },
        };
        let (_, b) = candidatas(c.modo)
            .into_iter()
            .find(|(n, _)| *n == quem)
            .unwrap();
        let (pior, fora) = desvio(&w3c_com(&f, c, b), c.px);
        assert!(
            pior <= 1,
            "GIMP {} {}%: a fórmula «{quem}» fica a {pior} degraus ({fora} canais)",
            c.nome,
            c.opacidade
        );
        // CONTROLO: a nossa fórmula diverge mesmo dele aqui (senão o modo não pertence à lista).
        assert!(desvio(&compoe(&f, c), c.px).0 > 4, "{}", c.nome);
        vistas += 1;
    }
    // Add, ColorBurn, ColorDodge, LinearBurn, LinearLight, SoftLight × 2 opacidades.
    assert_eq!(vistas, 12);
}
