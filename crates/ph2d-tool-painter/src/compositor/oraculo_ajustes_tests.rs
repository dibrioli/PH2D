//! ADR-0177, P3: um ajuste NÃO neutro de ecrã contra o GIMP «perceptual». A mesma grelha dos modos
//! (base + UMA camada, Normal 100) composta em tons de ecrã; o GIMP ajusta o visível
//! (`docs/Painter/ferramentas/oraculo_camadas_gimp/oraculo_ajustes.py`), nós empilhamos a camada de
//! ajuste por cima. Invert, Curves e Levels têm o espaço como parâmetro no GIMP: a corrida
//! «linear» é o CONTROLO (a régua distingue os dois espaços).

use super::*;
use crate::layers::LayerStack;
use ph2d_painter_effects::adjustments::{
    AdjustmentParams, ControlPoints, CurvesParams, InvertParams, LevelsParams, PosterizeParams,
    ThresholdParams,
};

static FIXTURA: &[u8] = include_bytes!(
    "../../../../docs/Painter/ferramentas/oraculo_camadas_gimp/gimp_3.2.6_ajustes.bin"
);

struct Corrida {
    ajuste: &'static str,
    espaco: &'static str,
    px: &'static [u8],
}

fn linha(at: &mut usize) -> &'static str {
    let fim = *at + FIXTURA[*at..].iter().position(|&b| b == b'\n').unwrap();
    let s = std::str::from_utf8(&FIXTURA[*at..fim]).unwrap();
    *at = fim + 1;
    s
}

/// `(w, h, base, topo, corridas)`.
fn le() -> (u32, u32, &'static [u8], &'static [u8], Vec<Corrida>) {
    let mut at = 0;
    let (mut w, mut h) = (0u32, 0u32);
    loop {
        let l = linha(&mut at);
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
    let mut bloco = |nome: &str| {
        assert_eq!(linha(&mut at), nome);
        at += n;
        &FIXTURA[at - n..at]
    };
    let (base, topo) = (bloco("BASE"), bloco("TOPO"));
    let mut corridas = Vec::new();
    while at < FIXTURA.len() {
        let mut c = linha(&mut at).split(' ');
        assert_eq!(c.next(), Some("RUN"));
        let (ajuste, espaco) = (c.next().unwrap(), c.next().unwrap());
        corridas.push(Corrida {
            ajuste,
            espaco,
            px: &FIXTURA[at..at + n],
        });
        at += n;
    }
    (w, h, base, topo, corridas)
}

/// Os NOSSOS parâmetros para o ajuste da corrida (os do `oraculo_ajustes.py`).
fn parametros(ajuste: &str) -> AdjustmentParams {
    match ajuste {
        "Invert" => AdjustmentParams::Invert(InvertParams {}),
        "Curves" => AdjustmentParams::Curves(CurvesParams {
            points_rgb: ControlPoints {
                points: vec![[0.0, 0.1037], [1.0, 0.9113]],
            },
            ..Default::default()
        }),
        "Levels" => AdjustmentParams::Levels(LevelsParams {
            black_point: 0.1,
            gamma: 1.6,
            white_point: 0.9,
            output_black: 0.05,
            output_white: 0.95,
        }),
        "Posterize" => AdjustmentParams::Posterize(PosterizeParams { levels: 4 }),
        "Threshold" => AdjustmentParams::Threshold(ThresholdParams { threshold: 128 }),
        _ => panic!("ajuste desconhecido na fixtura: {ajuste}"),
    }
}

/// O compositor de produção: base + camada (Normal 100) e, se houver, a camada de ajuste por cima.
fn compoe(w: u32, h: u32, base: &[u8], topo: &[u8], ajuste: Option<&str>) -> Vec<u8> {
    let mut s = LayerStack::new();
    let b = s.add_raster("base", w, h).unwrap();
    let t = s.add_raster("topo", w, h).unwrap();
    if let Some(nome) = ajuste {
        let p = parametros(nome);
        let a = s.add_adjustment(p.kind()).unwrap();
        if let Some(LayerKind::Adjustment(adj)) = s.get_mut(a).map(|l| &mut l.kind) {
            adj.params = p;
        }
    }
    let mut src = MapPixelSource::default();
    let img = |px: &[u8]| LayerImage {
        width: w,
        height: h,
        rgba8: px.to_vec(),
    };
    src.insert(b, img(base));
    src.insert(t, img(topo));
    composite(&s, &src, w, h)
}

/// O controlo: o mesmo ajuste EM LUZ (decode → `f` → encode) sobre o composto em VÍRGULA FLUTUANTE
/// da base e da camada (`Normal`, W3C straight em tons de ecrã) — o GIMP não quantiza entre os dois
/// passos, e no escuro a luz amplia um degrau do composto a 6.
fn em_luz(ajuste: &str, base: &[u8], topo: &[u8]) -> Vec<u8> {
    use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_unit};
    let f: fn(f32) -> f32 = match ajuste {
        "Invert" => |v| 1.0 - v,
        "Curves" => |v| 0.1037 + (0.9113 - 0.1037) * v,
        "Levels" => |v| 0.05 + ((v - 0.1) / 0.8).clamp(0.0, 1.0).powf(1.0 / 1.6) * 0.9,
        _ => unreachable!(),
    };
    let u = |b: u8| f32::from(b) / 255.0;
    let mut out = Vec::with_capacity(base.len());
    for (b, t) in base.as_chunks::<4>().0.iter().zip(topo.as_chunks::<4>().0) {
        let (ab, at) = (u(b[3]), u(t[3]));
        let ao = at + ab * (1.0 - at);
        for k in 0..3 {
            let co = if ao > 0.0 {
                (u(t[k]) * at + u(b[k]) * ab * (1.0 - at)) / ao
            } else {
                0.0
            };
            out.push(linear_to_srgb_byte(f(srgb_to_linear_unit(co))));
        }
        out.push((ao * 255.0).round() as u8);
    }
    out
}

/// `(pior desvio, canais a 1 degrau, canais comparados)` sobre os píxeis cujo índice `conta`
/// aceita; alfa 0 no GIMP = só o alfa; `alfa = false` compara só a cor.
fn desvio(
    nosso: &[u8],
    gimp: &[u8],
    alfa: bool,
    conta: impl Fn(usize) -> bool,
) -> (u8, usize, usize) {
    let (mut pior, mut um, mut n) = (0u8, 0usize, 0usize);
    let pares = nosso.as_chunks::<4>().0.iter().zip(gimp.as_chunks::<4>().0);
    for (i, (a, b)) in pares.enumerate() {
        if !conta(i) {
            continue;
        }
        let canais = match (b[3] == 0, alfa) {
            (true, _) => 3..4,
            (false, true) => 0..4,
            (false, false) => 0..3,
        };
        for k in canais {
            let d = a[k].abs_diff(b[k]);
            pior = pior.max(d);
            um += usize::from(d == 1);
            n += 1;
        }
    }
    (pior, um, n)
}

/// ⭐ Cada ajuste de ecrã NÃO neutro é o do GIMP «perceptual» (medido a 03/10: Invert, Posterize e
/// Threshold `0`; Curves `1` em 3 de 4 896 canais; Levels `2`, a tabela), e a régua distingue o
/// espaço: o controlo «linear» do GIMP é o mesmo ajuste em luz (`≤ 1`) e fica a `≥ 40` degraus do
/// nosso (Invert `120`, Curves `65`, Levels `60`). O Threshold do GIMP lê o canal VALUE num intervalo
/// `[low, high]` que DEIXA DE FORA o branco puro (`high` não passa de `1`): compara-se nos cinzentos
/// (onde VALUE = a luma Rec.601), e o branco é a divergência nomeada; o Posterize dele quantiza
/// também o alfa (compara-se a cor).
#[test]
fn um_ajuste_de_ecra_e_o_do_gimp_perceptual() {
    let (w, h, base, topo, corridas) = le();
    let sem = compoe(w, h, base, topo, None);
    let cinzento = |px: &[u8; 4]| px[0] == px[1] && px[1] == px[2];
    let mut vistos = Vec::new();
    for c in &corridas {
        let nosso = compoe(w, h, base, topo, Some(c.ajuste));
        // A Threshold escolhe pelo NOSSO composto sem ajuste (a entrada de cada píxel).
        let entrada = sem.as_chunks::<4>().0;
        let conta = |i: usize| {
            let e = &entrada[i];
            c.ajuste != "Threshold" || (cinzento(e) && e[0] < 255)
        };
        match c.espaco {
            "perceptual" | "-" => {
                // O Posterize do GIMP posteriza também o ALFA (a divergência nomeada, abaixo).
                let alfa = c.ajuste != "Posterize";
                let (pior, um, n) = desvio(&nosso, c.px, alfa, conta);
                eprintln!(
                    "{:<10} perceptual: pior {pior}, {um} de {n} canais a 1",
                    c.ajuste
                );
                assert!(n > 1_000, "{}: só {n} canais comparados", c.ajuste);
                // Levels: a tabela de 256 com interpolação (a que a placa liga) erra 2 junto do
                // ponto preto, onde `t^(1/γ)` é mais íngreme — a função exacta dá 0 (medido).
                let tecto = if c.ajuste == "Levels" { 2 } else { 1 };
                let fora = (0..entrada.len())
                    .filter(|&i| conta(i))
                    .find(|&i| (0..3).any(|k| nosso[i * 4 + k].abs_diff(c.px[i * 4 + k]) > tecto));
                assert!(
                    pior <= tecto,
                    "{} contra o GIMP «perceptual»: {pior} — 1.º píxel {fora:?}: entrada {:?} nosso {:?} gimp {:?}",
                    c.ajuste,
                    fora.map(|i| entrada[i]),
                    fora.map(|i| &nosso[i * 4..i * 4 + 4]),
                    fora.map(|i| &c.px[i * 4..i * 4 + 4]),
                );
                // Um gate ±1 não distingue arredondar de truncar: a fracção a 1 também é a medida.
                assert!(um * 100 <= n, "{}: {um} de {n} canais a 1", c.ajuste);
            }
            "linear" => {
                let (pior_nosso, _, _) = desvio(&nosso, c.px, true, |_| true);
                assert!(
                    pior_nosso >= 40,
                    "{}: a régua não distingue ({pior_nosso})",
                    c.ajuste
                );
                let (pior_luz, _, _) = desvio(&em_luz(c.ajuste, base, topo), c.px, true, |_| true);
                eprintln!(
                    "{:<10} linear (controlo): nosso a {pior_nosso}, a luz a {pior_luz}",
                    c.ajuste
                );
                assert!(
                    pior_luz <= 1,
                    "{}: o controlo «linear» não é a luz ({pior_luz})",
                    c.ajuste
                );
            }
            e => panic!("espaço desconhecido {e}"),
        }
        vistos.push(format!("{}/{}", c.ajuste, c.espaco));
    }
    assert_eq!(vistos.len(), 8, "{vistos:?}");
    // A divergência nomeada: o branco puro sai PRETO na Threshold do GIMP.
    let t = corridas.iter().find(|c| c.ajuste == "Threshold").unwrap();
    let brancos = sem
        .as_chunks::<4>()
        .0
        .iter()
        .zip(t.px.as_chunks::<4>().0)
        .filter(|(e, g)| cinzento(e) && e[0] == 255 && e[3] > 0 && g[0] == 0)
        .count();
    assert!(brancos > 0, "o GIMP passou a incluir o branco na Threshold");
    // … e o Posterize dele quantiza o ALFA (140 → 170 com 4 níveis); o nosso, como toda camada
    // de ajuste, muda a cor que o píxel tem e nunca a cobertura.
    let p = corridas.iter().find(|c| c.ajuste == "Posterize").unwrap();
    let alfas_mudados = sem
        .as_chunks::<4>()
        .0
        .iter()
        .zip(p.px.as_chunks::<4>().0)
        .filter(|(e, g)| e[3] == 140 && g[3] == 170)
        .count();
    assert!(
        alfas_mudados > 0,
        "o Posterize do GIMP deixou de mexer no alfa"
    );
}
