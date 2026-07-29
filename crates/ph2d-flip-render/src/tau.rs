//! **A LEI DA TINTA** — a integral de arco que o percurso binado resolve
//! ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md) §5 e §7.5, passo 3).
//!
//! ## ⚠️ Isto NÃO é uma lei nova. É a lei que o motor de hoje já usa, sobre a geometria CERTA.
//!
//! O `hardness_mask` do `flip.wgsl` compõe uma fileira de dabs por `over` e devolve
//! `1 − Π(1 − w_k)`. Tome o logaritmo do produto:
//!
//! ```text
//!   1 − α = Π (1 − w_k)   ⇒   −ln(1 − α) = Σ −ln(1 − w_k) = Σ f(d_k)   ⇒   α = 1 − exp(−τ)
//! ```
//!
//! O motor de hoje **já calcula `α = 1 − exp(−τ)`** — só que soma `τ` sobre uma **RETA
//! FICTÍCIA** que passa pelo ponto mais próximo (`d = √(dn² + along²)`, o `along` correndo por
//! uma fileira infinita e reta). É por isso que:
//!
//! - traço reto sai certo (medido: controle **+1/255**);
//! - o **cruzamento** sai errado — a ficção não tem cruzamento nenhum;
//! - a **ponta convexa** sai errada (**+140/255**) — a ficção tem caminho infinito onde o real
//!   acaba, e a nota do `hardness_law.rs` já a declarava superestimada.
//!
//! Aqui `τ` é integrado sobre o **CAMINHO QUE EXISTE**: `τ(p) = ∫ f(dn(s,p)) ds / pitch(s)`. Mesma
//! lei, geometria honesta — e, por ser uma SOMA, ela é **comutativa e sem teto**, que é exatamente
//! o que a lista por-ladrilho do [`crate::binning`] entrega (as propriedades (B) e (C) do §3).
//!
//! ## O que é `pitch`
//!
//! `PAINTER_SPACING × diâmetro` — o `spacing` default do pincel do Painter. ⚠️ **Não é dependência
//! de amostragem** (a doença que esta linha curou quatro vezes): é propriedade do PINCEL, não de
//! quão fino o motor amostrou o caminho. A §5.3 do doc 12 mediu o desvio **exatamente constante**
//! em 6 densidades de polilinha (60 → 1155 segmentos).

use crate::binning::{BinSeg, ScreenSpace};
use crate::pack::{FLAG_CLOSED, FLAG_START_FLAT, FlipGpuData};

/// O `spacing` do pincel do Painter (`spec_default.rs`): 0,10 × **diâmetro**.
pub const PAINTER_SPACING: f32 = 0.10;

/// Piso do pitch, em px — o mesmo do oráculo (`painter_look.rs::painter_deposit_sized`).
const MIN_PITCH_PX: f32 = 0.25;

/// Sub-amostras de quadratura por pitch. ⚠️ **4 SATURA** — medido na §5.4 do doc 12: de 4 para 8
/// o número não se move. Não é um palpite conservador, é onde a curva deitou.
pub const SUB: u32 = 4;

/// Teto de `f`. `f = −ln(1 − w)` **diverge** em `w → 1` (o disco duro de `hardness = 1`), e a
/// integral só precisa de um número grande o bastante para `1 − exp(−τ)` saturar em `f32`.
const F_MAX: f32 = 16.0;

/// A queda de **UM DAB** do Painter: platô até `hardness`, depois o preset `Falloff::Smooth`.
///
/// ⚠️ **É uma cópia por MOTOR, não uma cópia a mais.** O `flip.wgsl` carrega a dele (com o gate
/// `hardness_law.rs` provando termo a termo contra a função REAL do Painter); esta é a do motor
/// novo, com o gate irmão. Quando o motor velho sair, a cópia dele sai junto.
#[must_use]
pub fn dab_weight(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return f32::from(dn < 1.0);
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    if remapped >= 1.0 {
        return 0.0;
    }
    // `Falloff::Smooth` avaliado em `p = 1 − t` (convenção Blender), nas MESMAS operações e na
    // MESMA ordem que o `falloff.rs`. ⚠️ `p*p*(3−2p)` é a MESMA álgebra e **não é o mesmo `f32`** —
    // o gate `the_dab_weight_is_the_painters_falloff` nasceu vermelho exatamente nisso.
    let p = 1.0 - remapped;
    3.0 * p * p - 2.0 * p * p * p
}

/// **O PERFIL DE UM DAB deste traço** — o que decide a FORMA da queda, lido do traço UMA vez.
///
/// ⚠️ **Sem `Default`, e é o ponto.** O rasterizador tem SETE leitores de `Stroke`/flags e o
/// percurso tinha quatro: cada flag que fica de fora é uma feature apagada em silêncio, com o
/// motor novo armado e todos os gates verdes (foi o que a auditoria por grep achou depois do
/// smoke). Um par `(hardness, bool, bool, …)` solto convida a passar `false` onde o traço tinha a
/// resposta; um tipo sem `Default` obriga a construir do traço — a lei do `ShapeFrame` do Painter.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DabProfile {
    /// `0` = borda maximamente macia, `1` = disco duro (o default do Flip).
    pub hardness: f32,
    /// `FLAG_AIRBRUSH`: a transmitância de Beer-Lambert em vez do perfil do Painter.
    pub airbrush: bool,
}

impl DabProfile {
    /// Do traço — a porta única. Toda flag nova entra AQUI, e o compilador acha os leitores.
    #[must_use]
    pub fn of(st: &crate::pack::GpuStroke) -> Self {
        Self {
            hardness: st.hardness,
            airbrush: st.flags & crate::pack::FLAG_AIRBRUSH != 0,
        }
    }
}

/// Os limites do `k` do airbrush — **TÊM de bater com o `flip.wgsl`** (`AIRBRUSH_K_MIN/MAX`).
pub const AIRBRUSH_K_MIN: f32 = 1.0;
pub const AIRBRUSH_K_MAX: f32 = 8.0;

/// A densidade `f = −ln(1 − w)`. É ela que troca o **PRODUTO** por uma **SOMA** — e é a soma que
/// torna a lei comutativa, sem ordem e sem teto.
///
/// ⚠️ **NÃO use isto para o airbrush** — ele tem outra MEDIDA; a porta é a [`d_tau_of`].
#[must_use]
pub fn f_of(dn: f32, prof: DabProfile) -> f32 {
    let w = dab_weight(dn, prof.hardness);
    if w >= 1.0 {
        return F_MAX;
    }
    (-(1.0 - w).ln()).min(F_MAX)
}

/// O `k` de Beer-Lambert deste traço.
#[must_use]
pub fn airbrush_k(hardness: f32) -> f32 {
    AIRBRUSH_K_MIN + (AIRBRUSH_K_MAX - AIRBRUSH_K_MIN) * hardness.clamp(0.0, 1.0)
}

/// **O incremento de `τ` de uma sub-amostra — a porta ÚNICA, e ela decide a MEDIDA.**
///
/// Os dois perfis integram contra medidas DIFERENTES, e isso é a física, não arrumação:
///
/// - **padrão:** o pixel conta quantos DABS o cobriram (`step/pitch`), e cada um contribui a
///   densidade `−ln(1−w)`. Dabs são carimbos discretos compostos por `over`.
/// - **airbrush:** um spray deposita densidade por unidade de **CAMINHO**, não por dab ⇒ a medida
///   é `step/(2r)` (comprimento em diâmetros) e o `pitch` **cancela**.
///
/// ⚠️ **A densidade do airbrush é UNIFORME dentro do disco, e isso foi DERIVADO, não escolhido.**
/// O rasterizador escreve `w = 1 − exp(−k·√(1−dn²))` (Ciallo/Beer-Lambert), e esse `√(1−dn²)` é a
/// **projeção de Abel** da esfera — a corda pelo TUBO varrido, ou seja **já é a resposta do traço
/// inteiro**, não de um dab. A primeira tentativa desta wave usou `f = k·√(1−dn²)` por dab (o log e
/// o exp cancelam, o que é bonito e está errado) e a medição a matou: raster `252/251/249/242/192`
/// contra percurso `255/255/255/255/247` — integrar a corda ao longo do caminho a multiplica pelo
/// número de dabs. Invertendo a Abel, o kernel aditivo cuja integral de caminho **É** a corda é a
/// **indicadora do disco** (conferido numericamente a 4 decimais: `∫[√(y²+u²)<1] du = 2√(1−y²)`),
/// e a normalização sai da igualdade `C·2r·√(1−y²)/pitch = k·√(1−y²)` ⇒ `dτ = k·step/(2r)`.
///
/// ⚠️ **E o percurso fica MAIS correto que o rasterizador, não igual:** a corda analítica só vale
/// numa reta infinita; o percurso integra a densidade **ao longo do caminho de verdade**, então na
/// curva e no cruzamento ele responde o que a projeção fechada não sabe responder. Numa reta os
/// dois coincidem — é isso que o gate afirma.
#[must_use]
pub fn d_tau_of(dn: f32, prof: DabProfile, step: f32, r: f32, pitch: f32) -> f32 {
    if prof.airbrush {
        if dn >= 1.0 {
            return 0.0;
        }
        return airbrush_k(prof.hardness) * step / (2.0 * r);
    }
    let fv = f_of(dn, prof);
    if fv <= 0.0 {
        return 0.0;
    }
    fv * step / pitch
}

/// O que um traço deposita num pixel: `τ` acumulado e a cor média ponderada por `dτ`.
///
/// ⚠️ **A quadratura fica ANCORADA no segmento, não na janela do pixel.** As sub-amostras caem
/// sempre nas mesmas posições `(i + ½)·step`, e o que a janela faz é só **pular as que valem
/// zero**. Re-ancorar por pixel daria a cada um um erro de quadratura próprio — ruído sub-pixel
/// numa lei que existe para ser função pura do caminho.
///
/// O custo por segmento é **limitado, não proporcional ao comprimento**: só o arco dentro do disco
/// de raio `r` contribui, e ele mede no máximo `2r` ⇒ `2r / (pitch/SUB) = 8r/(0.2r) = 40`
/// amostras, independente de o segmento ter 5 px ou 5000.
pub(crate) fn stroke_tau(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    prof: DabProfile,
    p: [f32; 2],
) -> Option<(f32, [f32; 4])> {
    let mut tau = 0.0_f32;
    let mut acc = [0.0_f32; 4];
    for seg in run {
        let (pa, pb) = (data.points[seg.a as usize], data.points[seg.b as usize]);
        let sa = screen.point_px(pa.pos);
        let sb = screen.point_px(pb.pos);
        let ra = screen.radius_px(pa.width);
        let rb = screen.radius_px(pb.width);
        let v = [sb[0] - sa[0], sb[1] - sa[1]];
        let len2 = v[0] * v[0] + v[1] * v[1];
        if len2 <= 1e-12 {
            continue;
        }
        let len = len2.sqrt();

        // A janela: os `t` onde a amostra pode estar dentro do disco de raio `rmax` em torno de
        // `p`. `rmax` (e não o raio interpolado) mantém a janela CONSERVADORA — ela só pode
        // sobrar, nunca faltar.
        let rmax = ra.max(rb);
        let w = [sa[0] - p[0], sa[1] - p[1]];
        let wv = w[0] * v[0] + w[1] * v[1];
        let ww = w[0] * w[0] + w[1] * w[1];
        let disc = wv * wv - len2 * (ww - rmax * rmax);
        if disc <= 0.0 {
            continue;
        }
        let sq = disc.sqrt();
        let t0 = ((-wv - sq) / len2).clamp(0.0, 1.0);
        let t1 = ((-wv + sq) / len2).clamp(0.0, 1.0);
        if t1 <= t0 {
            continue;
        }

        // A grade da quadratura — do SEGMENTO, com o pitch mais apertado dele (conservador: mais
        // amostras, nunca menos).
        let pitch_min = (PAINTER_SPACING * 2.0 * ra.min(rb)).max(MIN_PITCH_PX);
        let ds = pitch_min / SUB as f32;
        let n = (len / ds).ceil().max(1.0);
        let step = len / n;
        let i0 = (t0 * len / step - 0.5).floor().max(0.0) as u32;
        let i1 = (t1 * len / step - 0.5).ceil().clamp(0.0, n - 1.0) as u32;

        for i in i0..=i1 {
            let t = ((i as f32 + 0.5) * step / len).clamp(0.0, 1.0);
            let s = [sa[0] + v[0] * t, sa[1] + v[1] * t];
            let r = (ra * (1.0 - t) + rb * t).max(1e-4);
            let dn = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt() / r;
            let pitch = (PAINTER_SPACING * 2.0 * r).max(MIN_PITCH_PX);
            let d_tau = d_tau_of(dn, prof, step, r, pitch);
            if d_tau <= 0.0 {
                continue;
            }
            tau += d_tau;
            // A cor é a média ponderada por `dτ` — a resposta comutativa, do mesmo tipo da lei.
            // ⚠️ O `opacity` multiplica DEPOIS da cobertura (a regra do GP que o `flip.wgsl`
            // documenta: *um traço a opacity 0,5 não escurece sobre si mesmo*), então ele entra
            // no alfa da COR e nunca no `f`.
            let op = pa.opacity * (1.0 - t) + pb.opacity * t;
            for (dst, (ca, cb)) in acc.iter_mut().zip(pa.color.iter().zip(&pb.color)).take(3) {
                *dst += (ca * (1.0 - t) + cb * t) * d_tau;
            }
            acc[3] += (pa.color[3] * (1.0 - t) + pb.color[3] * t) * op * d_tau;
        }
    }
    end_dab(run, data, screen, prof, p, &mut tau, &mut acc);
    if tau <= 0.0 {
        return None;
    }
    for c in &mut acc {
        *c /= tau;
    }
    Some((tau, acc))
}

/// **O CAP — e ele é UM TERMO DE FRONTEIRA, não uma geometria nova.**
///
/// O Painter **SOMA** dabs (`Σ_k g(k)`, a partir de um dab no primeiro ponto); nós **INTEGRAMOS**
/// (`∫ g du`). Euler-Maclaurin diz em que elas diferem:
///
/// ```text
///   Σ_{k=0}^{N} g(k) = ∫_0^N g(u) du + [g(0) + g(N)] / 2 + …
/// ```
///
/// ⚠️ **No MEIO do caminho os termos de fronteira estão no infinito, onde `g = 0`** — por isso a
/// soma e a integral já concordavam ali (o corpo do traço reto mede +1/255 contra o depósito, e
/// **+0** contra o motor que shipa em dureza 1). **Na PONTA o termo sobrevive**, e ele é
/// exatamente **meio dab**. Não há forma nova a desenhar: a silhueta redonda já vem do `t`
/// clampado do `closest_on_seg`, e o que faltava era este meio termo.
///
/// Medido, na região da ponta de um traço reto contra o depósito do Painter: **−66/255 (dureza
/// 0,4) · −102 (0,7) · −52 (0,2)**, média ~16/255 — e em `hardness = 1` a ponta **já estava boa**
/// (média 1,7/255 contra a ÁREA), porque ali a saturação carrega.
///
/// ⚠️ **Traço FECHADO não tem ponta** (`FLAG_CLOSED`) e ponta CHATA não tem meio dab
/// (`FLAG_START_FLAT`): um cap Flat corta a tinta no plano da ponta, e o termo de fronteira é
/// justamente o que a arredonda.
fn end_dab(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    prof: DabProfile,
    p: [f32; 2],
    tau: &mut f32,
    acc: &mut [f32; 4],
) {
    let Some(first) = run.first() else { return };
    let st = data.strokes[first.stroke as usize];
    if st.flags & FLAG_CLOSED != 0 || st.point_count == 0 {
        return;
    }
    // ⚠️ **SÓ NO COMEÇO, e a assimetria é da REFERÊNCIA, não uma escolha de estética.** O
    // depósito do Painter carimba um dab **exatamente** no primeiro ponto e depois anda por
    // `pitch`, então o percurso dele **acaba ANTES do último ponto**, num lugar que depende do
    // comprimento total: a fronteira do começo é exata (⇒ o termo `g(0)/2` sobrevive) e a do fim
    // é fracionária (⇒ o termo médio é ZERO). Medido nas DUAS pontas de um traço reto, erro médio
    // contra o depósito: com meio dab nos dois lados o FIM sai **13,6 · 14,9 · 12,7** (durezas
    // 0,4 · 0,7 · 0,2); só no começo, **1,2 · 1,8 · 1,3**. O começo fica igual (2,3 · 3,3 · 1,9).
    //
    // ⚠️ **A FORMA do cap continua simétrica** — a silhueta redonda vem do `t` clampado do
    // `closest_on_seg`, nas duas pontas. O que é assimétrico é a **correção de quadratura**, e ela
    // é invisível na geometria.
    //
    // ⚠️ **Ponto para o smoke:** o oráculo (`painter_deposit_sized`) não carimba dab de CAUDA no
    // pen-up, e o Painter do produto carimba. Se o Enio vir a ponta final fina demais, o termo do
    // fim volta — o número dele está medido aqui do lado.
    for (idx, flat) in [(st.first_point, st.flags & FLAG_START_FLAT != 0)] {
        if flat || idx as usize >= data.points.len() {
            continue;
        }
        let pt = data.points[idx as usize];
        let s = screen.point_px(pt.pos);
        let r = screen.radius_px(pt.width).max(1e-4);
        let dn = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt() / r;
        // ⚠️ **O airbrush não tem termo de fronteira.** Euler-Maclaurin corrige uma SOMA discreta
        // de dabs; a medida do airbrush é comprimento de caminho, e a integral sobre o caminho REAL
        // já é exata nas pontas — meio dab aqui seria uma correção para um erro que não existe.
        if prof.airbrush {
            return;
        }
        let d_tau = 0.5 * f_of(dn, prof);
        if d_tau <= 0.0 {
            continue;
        }
        *tau += d_tau;
        for (dst, c) in acc.iter_mut().zip(&pt.color).take(3) {
            *dst += c * d_tau;
        }
        acc[3] += pt.color[3] * pt.opacity * d_tau;
    }
}
