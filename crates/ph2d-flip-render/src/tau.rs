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
///
/// ⚠️ **Isto responde só à FORMA da queda.** *Onde os dabs estão* é a [`TipShape`], e as duas
/// viajam juntas no [`StrokeStyle`] — a porta que o compilador obriga a construir do traço.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DabProfile {
    /// `0` = borda maximamente macia, `1` = disco duro (o default do Flip).
    pub hardness: f32,
    /// `FLAG_AIRBRUSH`: a transmitância de Beer-Lambert em vez do perfil do Painter.
    pub airbrush: bool,
}

impl DabProfile {
    /// Do traço — a porta única para a FORMA da queda.
    #[must_use]
    pub fn of(st: &crate::pack::GpuStroke) -> Self {
        Self {
            hardness: st.hardness,
            airbrush: st.flags & crate::pack::FLAG_AIRBRUSH != 0,
        }
    }
}

/// Os códigos do *tip* — a tabela mora no `pack.rs::tip_code` (a porta única CPU→GPU) e estes são
/// os leitores dela. ⚠️ TÊM de bater com os `const TIP_*` do `flip.wgsl` e do `walk.wgsl`.
const TIP_DOTS: u32 = 1;
const TIP_SQUARES: u32 = 2;

/// **ONDE OS DABS ESTÃO** — a ponta ao longo do traço ([`ph2d_flip::StrokeTip`]), lida UMA vez.
///
/// ⚠️ **É uma pergunta DIFERENTE da do [`DabProfile`]**, e é por isso que o percurso não precisa de
/// kernel novo para o pincel pontilhado: um traço contínuo é a soma de dabs tão juntos que ela
/// converge para a integral de arco; uma fileira de contas é a MESMA soma, com os dabs longe um do
/// outro. Só a LISTA de dabs muda.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TipShape {
    /// Linha cheia: a silhueta é a FITA e `τ` é a integral de arco.
    Continuous,
    /// Contas: a silhueta é a UNIÃO dos discos e `τ` é a SOMA sobre eles.
    Beads {
        /// Centro-a-centro em unidades de MUNDO — `dot_spacing × ref_width`, o múltiplo do
        /// diâmetro que o `flip.wgsl` usa (pitch relativa à espessura, senão traço grosso funde
        /// as contas num borrão).
        pitch: f32,
        /// `StrokeTip::Squares`: a conta é um quadrado orientado à tangente, não um disco.
        square: bool,
    },
}

impl TipShape {
    /// Do traço — a porta única para as POSIÇÕES dos dabs.
    #[must_use]
    pub fn of(st: &crate::pack::GpuStroke) -> Self {
        if st.tip != TIP_DOTS && st.tip != TIP_SQUARES {
            return Self::Continuous;
        }
        let pitch = st.dot_spacing * st.ref_width;
        // ⚠️ **Conta mais junta que o dab do próprio pincel É a linha cheia**, e o limiar sai da
        // LEI, não de um palpite: a soma sobre dabs espaçados de `PAINTER_SPACING × diâmetro` é
        // exatamente a integral de arco (é a definição do `pitch` do §"O que é `pitch`"), então
        // abaixo disso a resposta honesta é a contínua. O raster concorda por outra via — com
        // `dot_spacing = 0` ele desliga o tip por conta própria, e em `dot_spacing` minúsculo as
        // contas se sobrepõem tanto que a fileira lê como linha.
        //
        // ⚠️ **E é isto que LIMITA o laço:** a janela de um pixel mede `2r` de arco, então a
        // contagem de contas nela é `2r/pitch ≤ 1/PAINTER_SPACING = 10` — sem cap escolhido a
        // dedo, sem contagem que dispara quando o slider vai a zero.
        // ⚠️ Na forma POSITIVA e negada depois, nunca `<=`: `NaN <= x` é falso, então a versão
        // direta deixaria uma pitch `NaN` passar para o laço de contas.
        let usavel = st.dot_spacing > PAINTER_SPACING && pitch > 0.0;
        if !usavel {
            return Self::Continuous;
        }
        Self::Beads {
            pitch,
            square: st.tip == TIP_SQUARES,
        }
    }
}

/// **O ESTILO deste traço** — tudo o que um pixel precisa saber do pincel, lido do traço UMA vez.
///
/// ⚠️ **Sem `Default`, e as duas metades viajam juntas de propósito.** Dois tipos soltos deixam um
/// chamador construir um e esquecer o outro, que é exatamente o modo de falha que a auditoria por
/// grep achou (cinco features apagadas em silêncio com todos os gates verdes). Aqui a `of` é uma, e
/// a próxima flag entra nela.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StrokeStyle {
    /// A forma da queda de um dab.
    pub profile: DabProfile,
    /// Onde os dabs estão.
    pub tip: TipShape,
}

impl StrokeStyle {
    /// Do traço — a porta única.
    #[must_use]
    pub fn of(st: &crate::pack::GpuStroke) -> Self {
        Self {
            profile: DabProfile::of(st),
            tip: TipShape::of(st),
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

/// **A densidade de UM CARIMBO a `dn`** — a medida por DAB, irmã da [`d_tau_of`] (que é por
/// CAMINHO). Um dab sozinho tem `α = 1 − exp(−f)`, e no perfil padrão isso devolve **exatamente**
/// `dab_weight` — a identidade que faz da soma a MESMA lei, e é ela que dá o pincel pontilhado de
/// graça.
///
/// ⚠️ **No airbrush a fórmula é a corda `k·√(1−dn²)` — a MESMA que a wave anterior mediu e
/// REPROVOU**, e as duas coisas são verdade: lá ela era integrada ao longo do caminho, o que a
/// multiplica pelo número de dabs; para UM carimbo ela é a resposta certa, e
/// `1 − exp(−k√(1−dn²))` é ao bit o que o `hardness_mask` do `flip.wgsl` escreve. As duas medidas
/// se reconciliam aqui: a corda é a projeção de Abel de UM dab, e a densidade uniforme do
/// [`d_tau_of`] é a inversão dela.
#[must_use]
pub fn f_bead_of(dn: f32, prof: DabProfile) -> f32 {
    if dn >= 1.0 {
        return 0.0;
    }
    if prof.airbrush {
        return airbrush_k(prof.hardness) * (1.0 - dn * dn).max(0.0).sqrt();
    }
    f_of(dn, prof)
}

/// **O ALCANCE de um dab a partir da LINHA-DE-CENTRO** — quantos pixels ao lado do caminho este
/// pincel consegue pôr tinta, dado o maior raio em jogo.
///
/// ⚠️ **Um carimbo QUADRADO alcança `r√2` na diagonal**, e essa é a razão de esta função existir: o
/// binner e a janela da quadratura TÊM de perguntar a mesma coisa, senão o ladrilho lista o
/// segmento e a janela o descarta — ou pior, o contrário. Foi exatamente esse vão que a paridade
/// CPU×device pegou nas quinas dos quadrados: **os dois motores tinham o MESMO buraco**, e o que
/// divergia era o ulp do `disc <= 0` no limiar (a GPU contrai em FMA). Um gate de paridade não pode
/// achar um buraco compartilhado; o que o denunciou foi ele ficar EM CIMA da fronteira.
#[must_use]
pub fn dab_reach(tip: TipShape, rmax: f32) -> f32 {
    match tip {
        TipShape::Beads { square: true, .. } => rmax * std::f32::consts::SQRT_2,
        _ => rmax,
    }
}

/// A geometria de UMA conta, em PIXELS de tela.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Bead {
    /// Centro (px) — um ponto da linha-de-centro, no arco `k · pitch`.
    pub c: [f32; 2],
    /// Meia-espessura LOCAL (px): o TAMANHO da conta segue a espessura de onde ela está; só a
    /// pitch é por-traço.
    pub r: f32,
    /// Tangente UNITÁRIA do segmento (px) — só o quadrado a usa.
    pub dir: [f32; 2],
}

/// As contas que este segmento POSSUI, `[k0, k1]` inclusive (vazio se `k0 > k1`).
///
/// ⚠️ **Meio-aberta no fim** (`arc_a ≤ k·pitch < arc_b`): uma conta que cai exatamente numa JUNÇÃO
/// tem de ter UM dono, senão ela entra na soma duas vezes e a junção fica mais escura. E o `arc_b`
/// do segmento é `arc_a + |b−a|` — nunca `arc_len[b]` —, o que faz o segmento de FECHO de um anel
/// medir certo (lá o `arc_len` do ponto seguinte voltou a zero); é a MESMA aritmética do
/// `pack.rs`, então nos segmentos interiores os dois números são o mesmo `f32`.
///
/// ⚠️ **`end_inclusive` é a conta da PONTA, e ela não é detalhe:** o último ponto de um traço ABERTO
/// não tem segmento seguinte para adotar a conta que cai exatamente ali, então sem a exceção o
/// carimbo da ponta **desaparece** sempre que o arco total é múltiplo da pitch — que é justamente o
/// caso de um traço reto autorado em números redondos. O raster a desenha (ele clampa o `sc` dentro
/// do segmento), e num traço FECHADO ela seria a conta 0 outra vez ⇒ a exceção pede `!closed`.
pub(crate) fn bead_range(arc_a: f32, arc_b: f32, pitch: f32, end_inclusive: bool) -> (i32, i32) {
    let k0 = (arc_a / pitch).ceil() as i32;
    let k1 = if end_inclusive {
        (arc_b / pitch).floor() as i32
    } else {
        (arc_b / pitch).ceil() as i32 - 1
    };
    (k0, k1)
}

/// O último ponto de um traço ABERTO — quem responde ao `end_inclusive` do [`bead_range`].
/// Devolve `None` num traço fechado ou vazio (lá nenhuma conta é da ponta).
pub(crate) fn tail_point(data: &FlipGpuData, run: &[BinSeg]) -> Option<u32> {
    let st = data.strokes[run.first()?.stroke as usize];
    (st.flags & FLAG_CLOSED == 0 && st.point_count > 0).then(|| st.first_point + st.point_count - 1)
}

/// As contas que podem alcançar um pixel cuja janela cobre o arco `[lo, hi]` — alargada de uma
/// conta em cada lado. Ela só pode SOBRAR: quem não alcança sai pelo `dn ≥ 1`.
pub(crate) fn bead_window(lo: f32, hi: f32, pitch: f32) -> (i32, i32) {
    ((lo / pitch).floor() as i32, (hi / pitch).ceil() as i32)
}

/// A conta `k` sobre o segmento `sa→sb` (px) cujo início está no arco `arc_a` e que mede `wlen` de
/// MUNDO. ⚠️ A fração de arco **é** a fração de tela porque a câmera é uniforme — a mesma premissa
/// que o `bead_point` do `flip.wgsl` declara.
pub(crate) fn bead_at(
    (sa, sb): ([f32; 2], [f32; 2]),
    (ra, rb): (f32, f32),
    (arc_a, wlen): (f32, f32),
    (k, pitch): (i32, f32),
) -> Bead {
    let f = if wlen > 1e-12 {
        ((k as f32 * pitch - arc_a) / wlen).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let v = [sb[0] - sa[0], sb[1] - sa[1]];
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let dir = if len > 1e-6 {
        [v[0] / len, v[1] / len]
    } else {
        [1.0, 0.0]
    };
    Bead {
        c: [sa[0] + v[0] * f, sa[1] + v[1] * f],
        r: (ra * (1.0 - f) + rb * f).max(1e-4),
        dir,
    }
}

/// O `dn` de uma conta: distância EUCLIDIANA ao CENTRO, normalizada pelo raio local — ou a
/// Chebyshev no frame da tangente, para o quadrado.
///
/// ⚠️ **É a distância ao PONTO, nunca `√(dn² + arco²)`:** o arco curva, e a métrica mista esticava
/// a conta numa banana ao longo da curva (o 2º report do Enio sobre o tip, registrado no
/// `flip.wgsl`).
pub(crate) fn bead_dn(p: [f32; 2], b: Bead, square: bool) -> f32 {
    let d = [p[0] - b.c[0], p[1] - b.c[1]];
    if square {
        let along = (d[0] * b.dir[0] + d[1] * b.dir[1]).abs();
        let across = (-d[0] * b.dir[1] + d[1] * b.dir[0]).abs();
        along.max(across) / b.r
    } else {
        (d[0] * d[0] + d[1] * d[1]).sqrt() / b.r
    }
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
    style: StrokeStyle,
    p: [f32; 2],
) -> Option<(f32, [f32; 4])> {
    let prof = style.profile;
    let tail = tail_point(data, run);
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

        // A janela: os `t` onde a amostra pode estar dentro do disco de raio [`dab_reach`] em torno
        // de `p`. O `rmax` (e não o raio interpolado) mantém a janela CONSERVADORA — ela só pode
        // sobrar, nunca faltar — e o alcance sai da porta única porque um carimbo QUADRADO chega
        // mais longe que o raio dele.
        let rmax = ra.max(rb);
        let reach = dab_reach(style.tip, rmax);
        let w = [sa[0] - p[0], sa[1] - p[1]];
        let wv = w[0] * v[0] + w[1] * v[1];
        let ww = w[0] * w[0] + w[1] * w[1];
        let disc = wv * wv - len2 * (ww - reach * reach);
        if disc <= 0.0 {
            continue;
        }
        let sq = disc.sqrt();
        let t0 = ((-wv - sq) / len2).clamp(0.0, 1.0);
        let t1 = ((-wv + sq) / len2).clamp(0.0, 1.0);
        if t1 <= t0 {
            continue;
        }

        // **AS CONTAS** — a soma sobre os carimbos que esta janela alcança, SEM peso de arco: uma
        // conta é UM dab, não um tubo varrido. Fora daqui a lei não muda uma linha.
        if let TipShape::Beads { pitch, square } = style.tip {
            let arc_a = data.arc_len[seg.a as usize];
            let dw = [pb.pos[0] - pa.pos[0], pb.pos[1] - pa.pos[1]];
            let wlen = (dw[0] * dw[0] + dw[1] * dw[1]).sqrt();
            let (o0, o1) = bead_range(arc_a, arc_a + wlen, pitch, tail == Some(seg.b));
            let (w0, w1) = bead_window(arc_a + t0 * wlen, arc_a + t1 * wlen, pitch);
            for k in o0.max(w0)..=o1.min(w1) {
                let bead = bead_at((sa, sb), (ra, rb), (arc_a, wlen), (k, pitch));
                let d_tau = f_bead_of(bead_dn(p, bead, square), prof);
                if d_tau <= 0.0 {
                    continue;
                }
                tau += d_tau;
                // A conta carrega UMA cor — a de onde ela está. É o que um carimbo é.
                let f = if wlen > 1e-12 {
                    ((k as f32 * pitch - arc_a) / wlen).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let op = pa.opacity * (1.0 - f) + pb.opacity * f;
                for (dst, (ca, cb)) in acc.iter_mut().zip(pa.color.iter().zip(&pb.color)).take(3) {
                    *dst += (ca * (1.0 - f) + cb * f) * d_tau;
                }
                acc[3] += (pa.color[3] * (1.0 - f) + pb.color[3] * f) * op * d_tau;
            }
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
    end_dab(run, data, screen, style, p, &mut tau, &mut acc);
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
    style: StrokeStyle,
    p: [f32; 2],
    tau: &mut f32,
    acc: &mut [f32; 4],
) {
    let prof = style.profile;
    let Some(first) = run.first() else { return };
    let st = data.strokes[first.stroke as usize];
    if st.flags & FLAG_CLOSED != 0 || st.point_count == 0 {
        return;
    }
    // ⚠️ **Uma fileira de CONTAS não tem termo de fronteira**: Euler-Maclaurin corrige uma soma
    // discreta trocada por integral, e ali a soma já é discreta. A conta `k = 0` cai exatamente no
    // arco 0, ou seja **no primeiro ponto** — o carimbo da ponta já está na lista.
    if !matches!(style.tip, TipShape::Continuous) {
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
