//! **Família `texturas`** — J4, catálogo K (texturas, elementos, carimbos). Receitas, as IMAGENS que os carimbos
//! levam (desenhadas aqui, procedurais — nenhuma copiada de app nenhum) e os portões da família.
//! O exame de cada um: `docs/Painter/Pinceis/exames/texturas.md`.

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // K-6 — a RETÍCULA do mangá (Krita «y) Screentones Regular», 40 px): pontos de tamanho FIXO numa grelha presa à
    // TELA (o grão `Tiled` Dots a 45°, ~7,8 px de passo, raio ~2 px), o pincel redondo recorta-a; o tamanho pela
    // pressão. O passo e o ponto não crescem com o pincel (a retícula é do papel, não do traço).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.8"),
            // A orla de 3 px (J3 `borda_px`): a 5 px o dab é quase só orla e a trama fica pálida (Krita 0,25 de pico ×
            // 0,67 sem ela); o estado que as revisões r1–r3 e r5 tomaram pelo original (r4/r6: 4 px e dureza 0,95 estreitam).
            ("borda_px", "3.0"),
            ("texture", HALFTONE),
        ],
        mao: &[
            ("pressureSize", 0.98),
            ("pressureSizeCurve", 3.0),
            ("pressureOpacity", 0.6),
            ("pressureOpacityCurve", 1.5),
        ],
        size_px: Some(40.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Texture", "Texture", "Halftone")
    },
    // K-5 — as ESTRELAS (Rebelle «Watercolor/Spatter/Star»): estrelas de cinco pontas NOSSAS ([`estrela`]) espalhadas
    // ao longo e através, de todos os tamanhos e rodadas ao acaso, translúcidas com a orla um pouco mais escura.
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("falloff", "\"Constant\""),
            ("borda_px", "0.0"),
            ("shape", IMAGEM),
            ("spray_count", "2"),
            ("spacing", "0.85"),
            ("jitter", "0.5"),
            ("jitter_scale", "0.6"),
            // O passo pelo diâmetro à pressão (J4): na pressão fraca o rasto de estrelinhas do Rebelle (revisões r4, r6).
            ("passo_pela_pressao", "true"),
            ("jitter_rotate", "1.0"),
            ("continuous_spacing", "false"),
            (
                "dab_jitter",
                r#"{"across": 0.9, "angle_per_stroke": 0.0, "count": 0.5, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.05, "particle": 0.8, "particle_max_px": 56.0, "roundness": 0.0}"#,
            ),
        ],
        // A pressão no tamanho e o tamanho = os do preset do Rebelle (`pressure_size_influence` 1, Size 108). O tecto
        // da estrela em px (J4 «Motor J4»): a 200 px o Rebelle espalha-as por ~234 px mas nenhuma passa de 56 px (a
        // caixa da imagem dele; as que passariam saem QUADRADAS — o defeito não se imita, o tamanho sim).
        mao: &[("pressureSize", 1.0), ("pressureSizeCurve", 1.0), ("pressureOpacity", 0.6)],
        size_px: Some(108.0),
        opacity: Some(0.6),
        texturas: &[("shape", estrela)],
        ..ReceitaMeio::digital("Stamp", "Stamp", "Stars")
    },
    // K-3 — a RELVA (Krita «z) Stamp Grass», MyPaint `long_grass`): tufos de lâminas que sobem do traço, três
    // desenhos nossos ([`tufo`]) sorteados pela mangueira (L19), quase de pé (±15°), a base que esmaece.
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("falloff", "\"Constant\""),
            ("borda_px", "0.0"),
            ("shape", IMAGEM_AREA),
            ("hose", r#"{"n": 3, "mode": 0, "pesos": [1, 1, 1, 1, 1, 1, 1, 1]}"#),
            ("spray_count", "1"),
            ("spacing", "0.75"),
            ("jitter", "0.06"),
            ("jitter_scale", "0.4"),
            ("jitter_rotate", "0.1"),
            ("continuous_spacing", "false"),
        ],
        mao: &[
            ("pressureSize", 0.1),
            ("pressureOpacity", 0.8),
            ("pressureOpacityCurve", 1.0),
        ],
        size_px: None,
        opacity: Some(1.0),
        texturas: &[("shape", tufo_0), ("shape_2", tufo_1), ("shape_3", tufo_2)],
        ..ReceitaMeio::digital("Stamp", "Stamp", "Grass")
    },
];

/// Os REPROVADOS da família (J4, `docs/Painter/Pinceis/exames/texturas.md`): a receita da última ronda, fora da
/// fábrica — as revisões cegas acharam o oráculo e listaram o que falta (o exame diz o quê, e a fila o que destrava).
pub const EM_ESTUDO: &[ReceitaMeio] = &[
    // K-2 — a FOLHAGEM (Krita «z) Stamp Leaves»): cada dab é um RAMINHO nosso ([`raminho`]: um pé, sete folhas e um
    // maciço no meio), sorteado em posição, tamanho, rotação e espelho; a pressão leva a opacidade (a rampa 0→1 do Krita
    // 0,165 → 0,341 de cobertura; a nossa 0,161 → 0,337 — J4 «Motor J4»).
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("falloff", "\"Constant\""),
            ("borda_px", "0.0"),
            ("shape", IMAGEM_AREA),
            ("spray_count", "5"),
            ("spacing", "0.08"),
            ("jitter", "0.3"),
            ("jitter_scale", "0.4"),
            ("jitter_rotate", "1.0"),
            ("continuous_spacing", "false"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.4, "flip_x": 0.5, "flip_y": 0.0, "opacity": 0.0, "particle": 0.42, "particle_min_px": 3.5, "roundness": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 0.8),
            ("pressureOpacityCurve", 1.0),
        ],
        // O tamanho do carimbo do Krita (svg) não se mede: o do artista.
        size_px: None,
        opacity: Some(1.0),
        texturas: &[("shape", raminho)],
        ..ReceitaMeio::digital("Stamp", "Stamp", "Foliage")
    },
    // K-4 — o BOKEH (Krita «z) Stamp Bokeh»): discos de luz espalhados, um ARO nítido (0,72) sobre um miolo
    // translúcido (0,20 — o perfil medido de um disco de 150 px), tamanhos muito variados; o tamanho pela pressão.
    // J4 «Motor J4»: o passo pelo diâmetro à pressão (o rasto de bolinhas); a pressão a crescer o disco pela curva
    // 1,5 (r10–r12: o Krita vai de pontinhos a discos grandes); sem micro-pontos (r12: «pontinhos escuros a mais»).
    ReceitaMeio {
        pincel: &[
            ("falloff", "\"Custom\""),
            ("custom_falloff", ARO),
            ("hardness", "0.0"),
            ("borda_px", "0.0"),
            ("spray_count", "3"),
            ("spacing", "0.75"),
            // O passo pelo diâmetro À PRESSÃO (J4 «Motor J4»): na pressão fraca o rasto denso de bolinhas do Krita.
            ("passo_pela_pressao", "true"),
            ("jitter", "0.6"),
            ("jitter_scale", "0.35"),
            ("continuous_spacing", "false"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.1, "particle": 0.6, "roundness": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 1.0), ("pressureSizeCurve", 1.5), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        size_px: None,
        opacity: Some(0.75),
        ..ReceitaMeio::digital("Stamp", "Stamp", "Bokeh")
    },
    // K-8 — o GRÃO SUJO (Krita «y) Texture Noise»): cada dab é um punhado de RISCOS nossos ([`riscos`]: traços
    // curtos e translúcidos em ângulos ao acaso), rodado ao acaso e espalhado — a massa nebulosa, de núcleo mais escuro
    // e borda esfiapada (revisão cega r1: os anéis do DistortedNoise liam-se como um padrão regular dentro de um tubo).
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("falloff", "\"Constant\""),
            ("borda_px", "0.0"),
            ("shape", IMAGEM_AREA),
            ("spacing", "0.12"),
            ("jitter", "0.05"),
            ("jitter_scale", "0.15"),
            ("jitter_rotate", "1.0"),
            ("continuous_spacing", "false"),
            ("accumulate", "true"),
            ("flow", "0.6"),
        ],
        mao: &[("pressureSize", 0.3), ("pressureOpacity", 0.7), ("pressureOpacityCurve", 1.0)],
        size_px: None,
        opacity: Some(1.0),
        texturas: &[("shape", riscos)],
        ..ReceitaMeio::digital("Texture", "Grain", "Noise Grit")
    },
];

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[];

/// A retícula: o grão Dots preso à tela (`Tiled`), a 45° (as filas desencontradas da trama), 256/33 ≈ 7,8 px de passo,
/// raio 0,12 + 0,32·0,46 ≈ 0,27 do passo (~2 px), a orla de um píxel de AA (macieza 0,3).
const HALFTONE: &str = r#"{"angle_deg": 45, "flow": false, "kind": "Dots", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.3, 0.7, 0.0, 0.5], "rake": false, "size": [33.0, 33.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// A Shape de IMAGEM (a silhueta do ficheiro, `shape`): uma imagem por dab, no plano do dab.
pub(crate) const IMAGEM: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Image", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// A Shape de IMAGEM lida pela ÁREA do píxel (J4 «Motor J4», `GrainLaw::area`): a 5 px a média da imagem, não pontos.
const IMAGEM_AREA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Image", "lei": {"area": true, "depth_jitter": 0.0, "depth_min": 0.0, "depth_pressure": false, "mode": 0}, "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O perfil do disco de bokeh (x = a distância ao centro ÷ o raio): miolo 0,28 do aro, o aro a 0,95 e a queda
/// em 1,5 % do raio — o disco de 150 px do Krita: 47 · 52–75 (o brilho de dentro) · 166 · 0 em 255.
const ARO: &str = r#"[{"handle": "Vector", "id": 0, "x": 0.0, "y": 0.28}, {"handle": "Vector", "id": 1, "x": 0.92, "y": 0.31}, {"handle": "Vector", "id": 2, "x": 0.965, "y": 1.0}, {"handle": "Vector", "id": 3, "x": 0.99, "y": 0.6}, {"handle": "Vector", "id": 4, "x": 1.0, "y": 0.0}]"#;

// ── As imagens (nossas, procedurais: só polinómios e raízes — o mesmo byte em toda máquina) ──────────────────────

/// O lado das imagens dos carimbos (px) e a sobre-amostragem do AA (por eixo).
const LADO: usize = 128;
const SOBRE: usize = 4;

/// Uma máscara `LADO²` (255 = tinta) de `cobertura(x, y)` ∈ [0, 1] em [−1, 1]² (y para BAIXO), a média de
/// `SOBRE²` amostras por píxel — no formato do ficheiro.
fn mascara(cobertura: impl Fn(f64, f64) -> f64) -> Vec<u8> {
    let mut lum = vec![0u8; LADO * LADO];
    let n = (SOBRE * SOBRE) as f64;
    for py in 0..LADO {
        for px in 0..LADO {
            let mut s = 0.0;
            for sy in 0..SOBRE {
                for sx in 0..SOBRE {
                    let x = ((px * SOBRE + sx) as f64 + 0.5) / (LADO * SOBRE) as f64 * 2.0 - 1.0;
                    let y = ((py * SOBRE + sy) as f64 + 0.5) / (LADO * SOBRE) as f64 * 2.0 - 1.0;
                    s += cobertura(x, y).clamp(0.0, 1.0);
                }
            }
            lum[py * LADO + px] = (s / n * 255.0).round() as u8;
        }
    }
    ph2d_brush_presets::escrever_textura(&lum, LADO as u32, LADO as u32)
}

/// Dentro de uma folha com a base em `(bx, by)`, a ponta na direcção unitária `(dx, dy)` e o comprimento `l` (o perfil
/// da folha: a meia-largura `0,85·l·√s·(1 − s)` com os dentes).
fn na_folha(x: f64, y: f64, (bx, by): (f64, f64), (dx, dy): (f64, f64), l: f64) -> bool {
    let (vx, vy) = (x - bx, y - by);
    let s = (vx * dx + vy * dy) / l;
    let perp = (vx * -dy + vy * dx).abs();
    let dente = 1.0 + 0.14 * ((s * 5.0).fract() - 0.5).abs() * 2.0;
    (0.0..=1.0).contains(&s) && perp < 0.85 * l * s.sqrt() * (1.0 - s) * dente
}

/// UM raminho (o carimbo da folhagem): um pé que sobe, sete folhas alternadas a ±45° dele (a de cima de pé) e um
/// maciço no meio (o carimbo do Krita é um aglomerado) — as direcções são as constantes de 45° (sem trigonometria).
fn raminho() -> Vec<u8> {
    const R: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let folhas: [((f64, f64), (f64, f64), f64); 7] = [
        ((0.0, 0.55), (-R, -R), 0.42),
        ((0.02, 0.35), (R, -R), 0.45),
        ((0.03, 0.12), (-R, -R), 0.45),
        ((0.04, -0.1), (R, -R), 0.42),
        ((0.04, -0.3), (-R, -R), 0.36),
        ((0.04, -0.45), (R, -R), 0.32),
        ((0.04, -0.5), (0.0, -1.0), 0.38),
    ];
    mascara(|x, y| {
        let pe = (-0.5..0.75).contains(&y) && (x - 0.04 * (0.9 - y) / 1.4).abs() < 0.012;
        let macico = x * x + (y - 0.05) * (y - 0.05) < 0.13 * 0.13;
        let folha = folhas.iter().any(|&(b, d, l)| na_folha(x, y, b, d, l));
        f64::from(u8::from(pe || folha || macico))
    })
}



/// Uma lâmina de relva: da base `(bx, 0,92)` à ponta `(tx, ty)`, a curva de Bézier quadrática pelo controlo
/// abaixo da ponta (sobe e dobra para a direita, em gancho), fina na base e aguda na ponta, a tinta escura na
/// base e a esmaecer para a ponta (`1 − 0,55·s`: o tufo do Krita, revisão cega r2), a orla macia.
#[derive(Clone, Copy)]
struct Lamina {
    bx: f64,
    tx: f64,
    ty: f64,
    w: f64,
    /// A tinta da lâmina (`1` cheia; as «fantasmas» translúcidas do tufo do Krita ~0,35).
    forca: f64,
}

/// Um tufo de lâminas pintado por discos ao longo de cada curva (o máximo da tinta, não a soma).
fn tufo(laminas: &[Lamina]) -> Vec<u8> {
    let m = LADO * SOBRE;
    let mut alta = vec![0.0f64; m * m];
    for l in laminas {
        let (x0, y0, x2, y2) = (l.bx, 0.92, l.tx, l.ty);
        let (x1, y1) = (l.bx + 0.15 * (l.tx - l.bx), 0.35 * (0.92 + l.ty) + 0.3 * l.ty);
        for k in 0..=400 {
            let s = f64::from(k) / 400.0;
            let u = 1.0 - s;
            let cx = u * u * x0 + 2.0 * u * s * x1 + s * s * x2;
            let cy = u * u * y0 + 2.0 * u * s * y1 + s * s * y2;
            // Fina na base, a mais larga a um terço, a ponta aguda.
            let r = l.w * (3.0 * s).min(1.0) * (1.0 - s) + 0.004;
            let tinta = l.forca * (0.55 + 1.35 * s).min(1.0) * (1.0 - 0.55 * s * s);
            let (ix, iy) = (
                ((cx + 1.0) / 2.0 * m as f64) as i64,
                ((cy + 1.0) / 2.0 * m as f64) as i64,
            );
            let rr = (r / 2.0 * m as f64).ceil() as i64;
            for dy in -rr..=rr {
                for dx in -rr..=rr {
                    let (qx, qy) = (ix + dx, iy + dy);
                    if qx < 0 || qy < 0 || qx >= m as i64 || qy >= m as i64 {
                        continue;
                    }
                    let px = (qx as f64 + 0.5) / m as f64 * 2.0 - 1.0;
                    let py = (qy as f64 + 0.5) / m as f64 * 2.0 - 1.0;
                    // A orla macia da lâmina (o carimbo do Krita a 200 px é translúcido e esbatido, não recortado).
                    let d = ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt();
                    let borda = ((r - d) / (0.35 * r)).clamp(0.0, 1.0);
                    if borda > 0.0 {
                        let a = &mut alta[qy as usize * m + qx as usize];
                        *a = a.max(tinta * borda);
                    }
                }
            }
        }
    }
    mascara(|x, y| {
        let qx = (((x + 1.0) / 2.0) * m as f64) as usize;
        let qy = (((y + 1.0) / 2.0) * m as f64) as usize;
        alta[qy.min(m - 1) * m + qx.min(m - 1)]
    })
}

/// Os tufos: `n` lâminas sorteadas por um gerador INTEIRO (o mesmo byte em toda máquina) — a base perto do meio,
/// a ponta alta e inclinada para fora, finas; três desenhos para a mangueira.
fn tufo_sorteado(semente: u64, n: usize) -> Vec<u8> {
    let mut e = semente;
    let mut u = || {
        e = e.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (e >> 40) as f64 / (1u64 << 24) as f64
    };
    let laminas: Vec<Lamina> = (0..n)
        .map(|_| {
            // J4 «Motor J4»: a moita do Krita — a base larga, as lâminas FINAS e curvadas para a direita, de
            // alturas várias, um terço pálidas (translúcidas) e de tinta sorteada, não duas.
            let bx = (u() - 0.5) * 1.2 - 0.05;
            let inclina = 0.0 + 0.8 * u();
            let ty = -0.3 - 0.55 * u();
            let forca = if u() < 0.25 { 0.3 + 0.3 * u() } else { 0.85 + 0.15 * u() };
            Lamina { bx, tx: (bx + inclina).clamp(-0.9, 0.9), ty, w: 0.025 + 0.025 * u(), forca }
        })
        .collect();
    tufo(&laminas)
}

fn tufo_0() -> Vec<u8> {
    tufo_sorteado(11, 30)
}

fn tufo_1() -> Vec<u8> {
    tufo_sorteado(23, 28)
}

fn tufo_2() -> Vec<u8> {
    tufo_sorteado(37, 32)
}

/// O ponto `(x, y)` dentro do polígono `v` (par-ímpar).
fn dentro(v: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut d = false;
    for i in 0..v.len() {
        let (a, b) = (v[i], v[(i + 1) % v.len()]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            d = !d;
        }
    }
    d
}

/// UMA estrela de cinco pontas de pé (raio de dentro 0,48 do de fora), a orla de ~6 % um pouco mais escura (a borda
/// da aguada). Os vértices são as constantes de 72°/36° (sem trigonometria: o mesmo byte em toda máquina).
fn estrela() -> Vec<u8> {
    const C: [(f64, f64); 10] = [
        (0.0, 1.0),
        (-0.587_785_252_3, 0.809_016_994_4),
        (-0.951_056_516_3, 0.309_016_994_4),
        (-0.951_056_516_3, -0.309_016_994_4),
        (-0.587_785_252_3, -0.809_016_994_4),
        (0.0, -1.0),
        (0.587_785_252_3, -0.809_016_994_4),
        (0.951_056_516_3, -0.309_016_994_4),
        (0.951_056_516_3, 0.309_016_994_4),
        (0.587_785_252_3, 0.809_016_994_4),
    ];
    let poli = |r: f64| -> Vec<(f64, f64)> {
        C.iter()
            .enumerate()
            .map(|(i, &(cx, cy))| {
                let k = if i % 2 == 0 { r } else { 0.48 * r };
                (k * cx, -k * cy) // a imagem tem o y para baixo: a ponta de cima fica em cima
            })
            .collect()
    };
    let (fora, miolo) = (poli(0.9), poli(0.8));
    mascara(|x, y| {
        if !dentro(&fora, x, y) {
            0.0
        } else if dentro(&miolo, x, y) {
            0.7
        } else {
            1.0
        }
    })
}

/// A distância de `p` ao segmento `a–b` e a fracção ao longo dele.
fn ao_segmento(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    let (qx, qy) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
    ((qx * qx + qy * qy).sqrt(), t)
}

/// UMA mancha de tinta salpicada: o miolo (um disco e lóbulos), raios que afinam até à ponta e gotículas soltas
/// além deles — tudo sorteado por um gerador INTEIRO e direcções normalizadas por raiz (sem trigonometria).
pub(crate) fn mancha() -> Vec<u8> {
    let mut e = 0x5EED_u64;
    let u = |e: &mut u64| {
        *e = e.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (*e >> 40) as f64 / (1u64 << 24) as f64
    };
    let direccao = |e: &mut u64| loop {
        let (x, y) = (u(e) * 2.0 - 1.0, u(e) * 2.0 - 1.0);
        let n = (x * x + y * y).sqrt();
        if n > 0.2 && n <= 1.0 {
            break (x / n, y / n);
        }
    };
    // A roseta (o Splats do Rebelle, revisões r1–r2): o corpo cheio com a borda ondulada (lóbulos na orla), raios
    // CURTOS e poucas gotículas.
    let mut discos = vec![(0.0, 0.0, 0.2)];
    let mut raios = Vec::new();
    for _ in 0..14 {
        let (dx, dy) = direccao(&mut e);
        discos.push((dx * 0.2, dy * 0.2, 0.05 + 0.03 * u(&mut e)));
    }
    for _ in 0..16 {
        let (dx, dy) = direccao(&mut e);
        let comprimento = 0.27 + 0.1 * u(&mut e);
        raios.push(((dx * comprimento, dy * comprimento), 0.035 + 0.02 * u(&mut e)));
    }
    for _ in 0..4 {
        let (dx, dy) = direccao(&mut e);
        let d = 0.36 + 0.15 * u(&mut e);
        discos.push((dx * d, dy * d, 0.012 + 0.012 * u(&mut e)));
    }
    mascara(|x, y| {
        let disco = discos
            .iter()
            .any(|&(cx, cy, r)| (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r);
        let raio = raios.iter().any(|&(b, w)| {
            let (d, t) = ao_segmento((x, y), (0.0, 0.0), b);
            d <= w * (1.0 - t)
        });
        f64::from(u8::from(disco || raio))
    })
}

// ── O portão por foto da família (J4, `docs/Painter/Pinceis/exames/texturas.md`) ─────────────────────────────────

/// A identidade de um carimbo numa FAIXA da cena `tamanhos` (x 120–880) — as contas de
/// `docs/Painter/Pinceis/exames/texturas_foto.py`: a cobertura `c = máx|px − 255| ÷ 232`; as manchas = componentes
/// 4-ligados de `c ≥ 0,3` com área ≥ 3 px.
#[derive(Clone, Copy, Debug)]
struct Faixa {
    /// A cobertura média da faixa.
    cob: f64,
    /// Manchas por 100 px de comprimento.
    pontos: f64,
    /// A área mediana das manchas (px).
    gota: f64,
    /// A mediana de altura ÷ largura da caixa das manchas com área ≥ 10 (as lâminas de pé da relva).
    alto: f64,
    /// p90 ÷ p50 da cobertura dos píxeis com `c ≥ 0,1` (o aro do bokeh sobre o miolo).
    contraste: f64,
    /// O desvio-padrão ÷ a média das áreas (os pontos iguais da retícula).
    cv_area: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Cob,
    Pontos,
    Gota,
    Alto,
    Contraste,
    CvArea,
}

impl M {
    fn de(self, f: &Faixa) -> f64 {
        match self {
            M::Cob => f.cob,
            M::Pontos => f.pontos,
            M::Gota => f.gota,
            M::Alto => f.alto,
            M::Contraste => f.contraste,
            M::CvArea => f.cv_area,
        }
    }
    /// A tolerância contra a foto: as contagens e as áreas são relativas (o sorteio muda de semente para semente).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Cob => 0.02 + 0.3 * foto,
            M::Pontos | M::Gota => 0.45 * foto.max(2.0),
            M::Alto | M::Contraste => 0.25 + 0.1 * foto,
            M::CvArea => 0.15 + 0.35 * foto,
        }
    }
}

const X0: usize = 120;
const X1: usize = 880;
/// As faixas do recto de 40 px e do de 200 px na cena `tamanhos`.
const FAIXAS: [(usize, usize); 2] = [(120, 260), (300, 580)];

fn faixa(px: &[u8], w: usize, (y0, y1): (usize, usize)) -> Faixa {
    let (lw, lh) = (X1 - X0, y1 - y0);
    let c: Vec<f64> = (0..lh * lw)
        .map(|i| {
            let (x, y) = (X0 + i % lw, y0 + i / lw);
            let p = &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
            p.iter().map(|&v| (255.0 - f64::from(v)).abs()).fold(0.0, f64::max) / CHEIO
        })
        .collect();
    let cob = c.iter().sum::<f64>() / c.len() as f64;
    let mut visto = vec![false; c.len()];
    let (mut areas, mut altos) = (Vec::new(), Vec::new());
    for i in 0..c.len() {
        if visto[i] || c[i] < 0.3 {
            continue;
        }
        visto[i] = true;
        let mut pilha = vec![i];
        let (mut n, mut xa, mut xb, mut ya, mut yb) = (0usize, lw, 0, lh, 0);
        while let Some(k) = pilha.pop() {
            let (x, y) = (k % lw, k / lw);
            n += 1;
            (xa, xb, ya, yb) = (xa.min(x), xb.max(x), ya.min(y), yb.max(y));
            let viz = [
                (x + 1 < lw).then(|| k + 1),
                (x > 0).then(|| k - 1),
                (y + 1 < lh).then(|| k + lw),
                (y > 0).then(|| k - lw),
            ];
            for v in viz.into_iter().flatten() {
                if !visto[v] && c[v] >= 0.3 {
                    visto[v] = true;
                    pilha.push(v);
                }
            }
        }
        if n >= 3 {
            areas.push(n as f64);
            if n >= 10 {
                altos.push((yb - ya + 1) as f64 / (xb - xa + 1) as f64);
            }
        }
    }
    let mediana = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    };
    let mut tinta: Vec<f64> = c.iter().copied().filter(|&v| v >= 0.1).collect();
    tinta.sort_by(f64::total_cmp);
    let q = |f: f64| {
        tinta
            .get(((f * tinta.len() as f64) as usize).min(tinta.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0.0)
    };
    let n = areas.len() as f64;
    let m = areas.iter().sum::<f64>() / n.max(1.0);
    let sd = (areas.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / n.max(1.0)).sqrt();
    Faixa {
        cob,
        pontos: n / (lw as f64 / 100.0),
        gota: mediana(&mut areas.clone()),
        alto: mediana(&mut altos),
        contraste: q(0.9) / q(0.5).max(1e-6),
        cv_area: if m > 0.0 { sd / m } else { 0.0 },
    }
}

const CHEIO: f64 = 255.0 - 23.0;

/// As FOTOS dos oráculos na cena `tamanhos` (A), medidas por `texturas_foto.py` (21/10): `[faixa do recto de 40,
/// faixa do de 200]`, e a ESCALA a que o nosso se pinta (a do `exame.sh`; 1 nos carimbos espalhados — a largura de
/// uma nuvem de marcas soltas mede UMA marca, não o pincel, ver o exame).
#[rustfmt::skip]
const FOTOS: [(&str, &str, f64, [Faixa; 2]); 6] = [
    ("Halftone", "krita y) Screentones Regular", 1.04, [
        Faixa { cob: 0.05, pontos: 72.76, gota: 14.0, alto: 1.0, contraste: 1.13, cv_area: 0.37 },
        Faixa { cob: 0.158, pontos: 338.29, gota: 15.0, alto: 1.25, contraste: 1.01, cv_area: 0.16 }]),
    ("Foliage", "krita z) Stamp Leaves", 0.794, [
        Faixa { cob: 0.155, pontos: 1.58, gota: 16.0, alto: 1.0, contraste: 1.01, cv_area: 3.29 },
        Faixa { cob: 0.373, pontos: 2.24, gota: 185.0, alto: 1.13, contraste: 1.0, cv_area: 3.85 }]),
    ("Grass", "krita z) Stamp Grass", 0.909, [
        Faixa { cob: 0.059, pontos: 10.53, gota: 15.0, alto: 1.48, contraste: 1.93, cv_area: 1.73 },
        Faixa { cob: 0.132, pontos: 10.13, gota: 20.0, alto: 1.66, contraste: 1.85, cv_area: 2.78 }]),
    ("Bokeh", "krita z) Stamp Bokeh", 1.0, [
        Faixa { cob: 0.031, pontos: 21.18, gota: 5.0, alto: 1.0, contraste: 1.89, cv_area: 2.99 },
        Faixa { cob: 0.078, pontos: 3.03, gota: 7.0, alto: 1.0, contraste: 2.04, cv_area: 2.32 }]),
    ("Stars", "rebelle WATERCOLOR/spatter_star", 1.0, [
        Faixa { cob: 0.041, pontos: 4.47, gota: 216.0, alto: 0.97, contraste: 1.21, cv_area: 0.77 },
        Faixa { cob: 0.028, pontos: 0.79, gota: 2683.0, alto: 1.0, contraste: 1.04, cv_area: 0.49 }]),
    ("Splatter", "rebelle WATERCOLOR/spatter_splats", 1.0, [
        Faixa { cob: 0.047, pontos: 8.42, gota: 49.0, alto: 1.0, contraste: 1.74, cv_area: 1.55 },
        Faixa { cob: 0.038, pontos: 9.08, gota: 6.0, alto: 1.0, contraste: 1.12, cv_area: 3.26 }]),
];

/// O que faz a identidade de cada um: (faixa 0 = 40 px · 1 = 200 px, a medida). A estrela do Rebelle a 200 px é um
/// QUADRADO (o carimbo dele ao tecto), fora; o salpico a 200 px parte-se em gotículas que a régua das manchas conta.
/// Só os APROVADOS; as outras fotos da família ficam como IRMÃOS (a lei dos irmãos mede contra todas).
const IDENTIDADE: [(&str, &[(usize, M)]); 3] = [
    ("Halftone", &[(1, M::Pontos), (1, M::CvArea), (1, M::Contraste), (0, M::Pontos)]),
    ("Stars", &[(0, M::Gota), (0, M::Contraste), (0, M::CvArea)]),
    ("Grass", &[(1, M::Alto), (1, M::Contraste), (0, M::Pontos)]),
];

/// ⭐ **Cada carimbo reconhece-se pela foto do seu oráculo** (J4, `docs/Painter/Pinceis/exames/texturas.md`): pintado
/// na cena `tamanhos` da bateria à escala do exame, cada medida da sua identidade cai na tolerância da foto, e — a
/// lei dos IRMÃOS — fica estritamente mais perto (em tolerâncias somadas) da SUA foto que da de cada um dos outros.
#[test]
fn cada_carimbo_se_reconhece_pela_foto() {
    use crate::exame_bancada::{bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let tamanhos = bat["cenas"].as_array().unwrap().iter().find(|c| c["nome"] == "tamanhos").unwrap();
    let medidas: Vec<[Faixa; 2]> = std::thread::scope(|s| {
        let hs: Vec<_> = IDENTIDADE
            .iter()
            .map(|&(nome, _)| {
                let escala = FOTOS.iter().find(|f| f.0 == nome).unwrap().2;
                let mut cena = tamanhos.clone();
                for op in cena["ops"].as_array_mut().unwrap() {
                    if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
                        op["size_px"] = serde_json::json!(d * escala);
                    }
                }
                s.spawn(move || {
                    let p = pinceis(nome).remove(0);
                    let px = pinta_cena(&p, None, &cena, tela, papel, 0).1;
                    FAIXAS.map(|f| faixa(&px, tela[0] as usize, f))
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (k, &(nome, ms)) in IDENTIDADE.iter().enumerate() {
        let (_, oraculo, _, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
        let eu = &medidas[k];
        println!("[texturas] {nome}: {eu:?}\n           foto {oraculo}: {foto:?}");
        for &(f, m) in ms {
            let (v, alvo) = (m.de(&eu[f]), m.de(&foto[f]));
            if (v - alvo).abs() > m.tol(alvo) {
                falhas.push(format!("{nome}: {m:?}[{f}] {v:.3} fora da foto {alvo:.3} ± {:.3} ({oraculo})", m.tol(alvo)));
            }
        }
        let distancia = |outra: &[Faixa; 2]| -> f64 {
            ms.iter().map(|&(f, m)| (m.de(&eu[f]) - m.de(&outra[f])).abs() / m.tol(m.de(&foto[f]))).sum()
        };
        let propria = distancia(foto);
        for (outro, _, _, f2) in FOTOS.iter().filter(|f| f.0 != nome) {
            let d = distancia(f2);
            if propria >= d {
                falhas.push(format!("{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — pinta o irmão"));
            }
        }
    }
    // A relva DE PÉ (J4, o que a faz relva): a 200 px as manchas mais altas que largas como as do Krita (1,66); a relva
    // a rodar ao acaso dá 1,25 (a mutação G1 do exame). A tolerância geral do Alto (0,42) deixava-a passar.
    let k = IDENTIDADE.iter().position(|i| i.0 == "Grass").unwrap();
    if medidas[k][1].alto < 1.5 {
        falhas.push(format!("Grass: Alto[1] {:.2} < 1,5 — as lâminas não estão de pé (Krita 1,66)", medidas[k][1].alto));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Os reprovados não estão na fábrica (J3): nenhum ficheiro com o nome de uma receita `EM_ESTUDO` desta família nem
/// dos dois do aerógrafo que ela reexaminou (Splatter, Stipple) — e cada um continua a ser um preset que se gera.
#[test]
fn os_em_estudo_nao_estao_na_fabrica() {
    use crate::presets_de_fabrica::{parametros_do_meio, relativo_do_meio};
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    for r in EM_ESTUDO.iter().chain(super::aerografo::EM_ESTUDO) {
        let rel = relativo_do_meio(r);
        assert!(!raiz.join(&rel).exists(), "{}: em estudo mas na fábrica ({rel})", r.nome);
        let (p, t) = parametros_do_meio(r);
        assert_eq!(p.name, r.nome);
        // As imagens que a receita desenha chegam ao preset, cada uma pelo seu id.
        for (id, _) in r.texturas {
            assert!(t.contains_key(*id), "{}: a imagem {id} não chegou ao preset", r.nome);
        }
    }
}

/// UM punhado de grão sujo (o carimbo do Noise Grit; J4 «Motor J4», a revisão r5: o Krita são «rabiscos curvos,
/// grossos e esbatidos», não palha recta): MANCHAS largas e pálidas de orla macia + RABISCOS finos e escuros que
/// curvam (polilinhas que rodam um pouco a cada passo), espalhados por igual no disco — sorteados por um gerador
/// INTEIRO; as rotações por polinómios (cos ≈ 1 − t²/2, sen ≈ t − t³/6) e raízes, sem trigonometria.
fn riscos() -> Vec<u8> {
    let mut e = 0x6E01_5E_u64;
    let mut u = move || {
        e = e.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (e >> 40) as f64 / (1u64 << 24) as f64
    };
    let no_disco = |r: f64, u: &mut dyn FnMut() -> f64| loop {
        let (x, y) = (u() * 2.0 - 1.0, u() * 2.0 - 1.0);
        if x * x + y * y <= 1.0 {
            break (x * r, y * r);
        }
    };
    let direccao = |u: &mut dyn FnMut() -> f64| loop {
        let (x, y) = (u() * 2.0 - 1.0, u() * 2.0 - 1.0);
        let n = (x * x + y * y).sqrt();
        if n > 0.2 && n <= 1.0 {
            break (x / n, y / n);
        }
    };
    // (a, b, meia-largura, tinta, a fracção da meia-largura que é orla)
    let mut traco: Vec<((f64, f64), (f64, f64), f64, f64, f64)> = Vec::new();
    for _ in 0..18 {
        let (cx, cy) = no_disco(0.62, &mut u);
        let (dx, dy) = direccao(&mut u);
        let l = 0.1 + 0.2 * u();
        traco.push(((cx - dx * l, cy - dy * l), (cx + dx * l, cy + dy * l), 0.06 + 0.07 * u(), 0.2 + 0.22 * u(), 0.8));
    }
    for _ in 0..12 {
        let mut p = no_disco(0.62, &mut u);
        let mut d = direccao(&mut u);
        let curva = (u() - 0.5) * 1.4;
        let (w, tinta) = (0.01 + 0.007 * u(), 0.6 + 0.3 * u());
        for _ in 0..8 {
            let t = curva + (u() - 0.5) * 0.3;
            let (c, sn) = (1.0 - t * t / 2.0, t - t * t * t / 6.0);
            let r = (d.0 * c - d.1 * sn, d.0 * sn + d.1 * c);
            let n = (r.0 * r.0 + r.1 * r.1).sqrt();
            d = (r.0 / n, r.1 / n);
            let q = (p.0 + d.0 * 0.04, p.1 + d.1 * 0.04);
            traco.push((p, q, w, tinta, 0.6));
            p = q;
        }
    }
    mascara(|x, y| {
        // As marcas somam-se como tinta translúcida (onde se cruzam escurece).
        let fica = traco.iter().fold(1.0, |fica, &(a, b, w, tinta, orla)| {
            let s = ((w - ao_segmento((x, y), a, b).0) / (w * orla)).clamp(0.0, 1.0);
            fica * (1.0 - tinta * s * s * (3.0 - 2.0 * s))
        });
        1.0 - fica
    })
}

/// Escreve os `EM_ESTUDO` (desta família e os do aerógrafo que ela reexamina) em `$PH2D_EXAME_DIR/estudo/<relativo>`:
/// a bancada lê-os pelo caminho (`exame.sh <caminho>`) sem eles entrarem na fábrica. `PH2D_REGERA_SO` filtra.
#[test]
#[ignore = "bancada: PH2D_EXAME_DIR=… cargo test … familias::texturas::escreve_os_em_estudo -- --ignored"]
fn escreve_os_em_estudo() {
    use crate::presets_de_fabrica::{preset_do_meio, relativo_do_meio};
    let dir = std::path::PathBuf::from(std::env::var("PH2D_EXAME_DIR").expect("PH2D_EXAME_DIR")).join("estudo");
    let so = std::env::var("PH2D_REGERA_SO").ok();
    for r in EM_ESTUDO.iter().chain(super::aerografo::EM_ESTUDO) {
        if so.as_deref().is_some_and(|s| !s.split(',').any(|n| n == r.nome)) {
            continue;
        }
        let rel = relativo_do_meio(r);
        ph2d_brush_presets::gravar(&dir, &rel, &preset_do_meio(r)).unwrap();
        println!("[estudo] {}", dir.join(&rel).display());
    }
}
