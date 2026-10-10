//! **A grade de luz do quadro** — ancorada no mundo (G11), com dimensões que só dependem da
//! JANELA (o zoom não realoca).
//!
//! - O texel é uma potência de dois em metros: `texel = 2^⌈log₂(c0_px / ppm)⌉`. Dentro de uma
//!   oitava de zoom ele não muda (o texel no ecrã vai de `c0_px` a `2·c0_px`); só a passagem de
//!   oitava troca a rede.
//! - As dimensões cobrem a vista no PIOR caso da oitava (`c0_px` por texel) mais a margem de cada
//!   lado e um bloco de folga para a âncora — logo dependem só da janela e do degrau.
//! - A origem encaixa num múltiplo do espaçamento da cascata de TOPO (`2^(n−1)` texels): toda sonda
//!   de toda cascata cai nos MESMOS pontos do mundo de um quadro para o outro, e o pan não treme.

use kurbo::{Affine, Point};
use ph2d_light2d_gpu::{Config, Grade};

/// O degrau da luz no ecrã. ⚠️ Escolhido por MEDIÇÃO — a tabela está no handoff W3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Degrau {
    /// O lado de um texel em píxeis no zoom mais fino da oitava.
    pub c0_px: f64,
    /// A margem fora da vista, em fração da vista, de CADA lado (emissores fora da tela iluminam
    /// dentro dela).
    pub margem: f64,
    /// Píxeis do alvo do Vello por texel da grade na camada do vetor que bloqueia (a C0 lê-a EXATA:
    /// cada subamostra cai num píxel cuja área já é a cobertura).
    pub vetor_px_por_texel: u32,
    /// ⭐ O lado de um texel da grade FINA (a luz direta, C3) em píxeis no zoom mais fino da oitava
    /// — independente do nível (W4b). Mais grosso que o `c0_px` não afina nada (a grade fina é a
    /// das cascatas).
    pub fino_px: f64,
    /// ⭐ O lado de um texel da captura FINA dos opacos (onde a luz direta marcha, W5a) em píxeis no
    /// zoom mais fino da oitava. Mais grosso que o `c0_px`, a captura é a das cascatas.
    pub captura_px: f64,
    /// ⭐ W5c: a CASCATA HOLOGRÁFICA (HRC) no lugar das cascatas de Sannikov — ver o
    /// [`DEGRAU_PADRAO`].
    pub holografica: bool,
}

/// O degrau por omissão — MEDIDO (rodada W3, RTX 5060 Ti, 1920×1080; tabela no handoff W3): o K7
/// (≤ 3 GB/s a 60 Hz) só passa com o texel de 4 px e margem ≤ 12,5 % — `2,96 GB/s`, `49 MB/quadro`,
/// `2,32 ms` na placa. O texel de 2 px com margem de 25 % dava `19 GB/s` e `11 ms`. O recurso que
/// limita é a BANDA de memória; em janelas maiores o governador (W5) sobe o `c0_px`. Camada do vetor:
/// 1 px por texel, lida EXATA (o píxel do Vello já é a cobertura de área), mede `0,0012` contra o
/// polígono a `0,34 ms` (20 caminhos); lida bilinear, a mesma camada media `0,066`.
///
/// ⭐ W5a — a captura FINA dos opacos de ½ px (rodada `medida_luz2d_w5a`, nível Baixo): a borda da
/// sombra sai do lugar −0,79 px no contorno de 50 % (a das cascatas −4,21; a de 1 px −1,16) e a barra
/// de 5 cm faz a sombra do gabarito (massa 2,6 % · fundo 0,027; a das cascatas não a via; a de 1 px
/// alargava a lasca 11 %), por 0,06 ms de captura, +65 MB e o Vello da camada do vetor a 8 px por
/// texel (0,50 ms contra 0,23).
///
/// ⭐ W5c — a CASCATA HOLOGRÁFICA em todo nível (rodada `medida_luz2d_w5c`, 1920×1080, vista de
/// fábrica, `load` 0,8; tabela no handoff W5c): o rebote da cena 3 contra o gabarito Baixo · Médio ·
/// Alto 5,5 · 24 · 39 % → 2,1 · 2,2 · 1,0 %; a luz junto a uma face até 21 % → ≤ 1,1 %; o tempo
/// 2,8–4,5 · 6,9–9,8 · 22,4–31,0 → 1,8–3,8 · 5,1–6,7 · 26,3–28,4 ms; a memória 109 · 146 · 244 →
/// 148 · 309 · 894 MB (o recurso que ela paga). O limite medido: a luz menor que ~3 células de raio
/// (−20 % de viés no raio 1,5; os vaga-lumes da cena 2 têm ~2 no Baixo e saem a −2,7 %).
pub const DEGRAU_PADRAO: Degrau = Degrau {
    c0_px: 4.0,
    margem: 0.125,
    vetor_px_por_texel: 1,
    fino_px: 1.0,
    captura_px: 0.5,
    holografica: true,
};

/// O texel de `px` píxeis no zoom mais fino da oitava: a potência de 2 em metros (G11).
fn texel_de(px: f64, ppm: f64) -> f64 {
    (px / ppm).log2().ceil().exp2()
}

/// Texels de `px` píxeis por texel das cascatas: a razão entre os dois texels, ambos potências de 2
/// em metros — herdam a âncora e a oitava da grade.
fn por_texel_das_cascatas(mundo_para_tela: Affine, degrau: Degrau, px: f64) -> u32 {
    let ppm = mundo_para_tela.determinant().abs().sqrt();
    if !(ppm.is_finite() && ppm > 0.0) {
        return 1;
    }
    let razao = texel_de(degrau.c0_px, ppm) / texel_de(px, ppm);
    (razao.round() as u32).max(1)
}

/// ⭐ **Texels finos por texel das cascatas** (`Config::fina`) para a vista.
#[must_use]
pub fn fina_do_quadro(mundo_para_tela: Affine, degrau: Degrau) -> u32 {
    por_texel_das_cascatas(mundo_para_tela, degrau, degrau.fino_px)
}

/// ⭐ **Texels da captura fina dos opacos por texel das cascatas** (`Config::captura`, W5a).
#[must_use]
pub fn captura_do_quadro(mundo_para_tela: Affine, degrau: Degrau) -> u32 {
    por_texel_das_cascatas(mundo_para_tela, degrau, degrau.captura_px)
}

/// O afim píxel → mundo: o INVERSO do mundo → tela que o Vello usou, nas duas linhas que o
/// `receber.wgsl` lê.
#[must_use]
pub fn tela_para_mundo(mundo_para_tela: Affine) -> [[f64; 3]; 2] {
    let [a, b, c, d, e, f] = mundo_para_tela.inverse().as_coeffs();
    [[a, c, e], [b, d, f]]
}

/// A grade deste quadro para a vista `mundo_para_tela` sobre um alvo de `alvo_px` píxeis.
///
/// `None` com uma câmera degenerada (escala zero ou não finita).
#[must_use]
pub fn grade_do_quadro(
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
    degrau: Degrau,
) -> Option<Grade> {
    let ppm = mundo_para_tela.determinant().abs().sqrt();
    if !(ppm.is_finite() && ppm > 0.0) || alvo_px.0 == 0 || alvo_px.1 == 0 {
        return None;
    }
    let texel_m = texel_de(degrau.c0_px, ppm);
    let lado = |px: u32| (f64::from(px) / degrau.c0_px * (1.0 + 2.0 * degrau.margem)).ceil() as u32;
    let base = (lado(alvo_px.0).max(1), lado(alvo_px.1).max(1));

    // As dimensões efetivas com um bloco de folga — o bloco depende das dimensões (o número de
    // cascatas), então itera até parar.
    let efetiva = |w: u32, h: u32| {
        let g = Grade {
            origem_m: [0.0, 0.0],
            texel_m,
            largura: w,
            altura: h,
        };
        let cfg = Config::padrao(g);
        (cfg.grade_efetiva(), 1u32 << (cfg.n_cascatas() - 1))
    };
    let mut bloco = efetiva(base.0, base.1).1;
    let mut g = loop {
        let (g, b) = efetiva(base.0 + bloco, base.1 + bloco);
        if b == bloco {
            break g;
        }
        bloco = b;
    };

    // A origem: centrada na vista e ENCAIXADA na rede do bloco (arredondada, logo a folga de um
    // bloco cobre os dois lados).
    let centro = mundo_para_tela.inverse()
        * Point::new(f64::from(alvo_px.0) / 2.0, f64::from(alvo_px.1) / 2.0);
    let passo = f64::from(bloco) * texel_m;
    let encaixa = |c: f64, n: u32| ((c - f64::from(n) * texel_m / 2.0) / passo).round() * passo;
    g.origem_m = [encaixa(centro.x, g.largura), encaixa(centro.y, g.altura)];
    Some(g)
}

/// ⭐ **A região da JANELA na grade `g`** (a fina): `[início, fim)` em texels, com um de folga
/// de cada lado para a bilinear do receptor. Só ela se calcula — a grade cobre o pior zoom da oitava
/// e a margem (na vista de fábrica a 1920×1080, 82 % dela fica fora da janela, W4b).
#[must_use]
pub fn regiao_da_janela(
    g: Grade,
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
) -> ([u32; 2], [u32; 2]) {
    let inv = mundo_para_tela.inverse();
    let (w, h) = (f64::from(alvo_px.0), f64::from(alvo_px.1));
    let cantos = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)].map(|(x, y)| inv * Point::new(x, y));
    let eixo = |k: usize, n: u32| {
        let v = cantos.map(|p| {
            let c = if k == 0 { p.x } else { p.y };
            (c - g.origem_m[k]) / g.texel_m
        });
        let lo = v.iter().copied().fold(f64::INFINITY, f64::min).floor() - 1.0;
        let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max).ceil() + 1.0;
        let preso = |x: f64| x.clamp(0.0, f64::from(n)) as u32;
        (preso(lo), preso(hi))
    };
    let ((x0, x1), (y0, y1)) = (eixo(0, g.largura), eixo(1, g.altura));
    ([x0, y0], [x1, y1])
}
