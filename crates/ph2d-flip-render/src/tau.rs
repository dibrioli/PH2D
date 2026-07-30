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
use crate::dabs::{bead_at, bead_dn, bead_range, bead_window, dab_reach, seg_window, tail_point};
use crate::pack::{FLAG_CLOSED, FLAG_START_FLAT, FlipGpuData};

/// O `spacing` do pincel do Painter (`spec_default.rs`): 0,10 × **diâmetro**.
pub const PAINTER_SPACING: f32 = 0.10;

/// Piso do pitch, em px — o mesmo do oráculo (`painter_look.rs::painter_deposit_sized`).
const MIN_PITCH_PX: f32 = 0.25;

/// Sub-amostras de quadratura por pitch. ⚠️ **4 SATURA por cima e MORDE por baixo.**
///
/// De 4 para 8 o número não se move (§5.4 do doc 12), e **2 foi construído, medido e REPROVADO**: ele
/// compra **−30% do device** (2,73 → 1,90 ms a 200 traços em 1080p) e custa **o DOBRO do erro na
/// TAMPA** de um traço reto contra o depósito do Painter (−53 → −134) mais a queda do árbitro do
/// cruzamento de **11,7× para 7,1×**, abaixo da barra do gate de controle
/// (`the_new_engine_leaves_the_hard_default_where_the_shipping_engine_put_it`).
///
/// ⚠️ **E a tabela da §5.4 mediu numa QUINA (`h = 0,4`)** e concluiu *"4 satura"*: a quadratura **não
/// dói na quina, dói na TAMPA**, onde vive o termo de fronteira do §13. A saturação era real e a
/// conclusão era limitada pela fixture — 4 é o piso, não o conforto.
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

/// **O FADE SUB-PIXEL de UM DAB** — `smoothstep(0, 1, espessura_px)`, o `gpencil_frag.glsl:534` que
/// o `flip.wgsl` multiplica na máscara.
///
/// ⚠️ **Ele é o PAR do piso de largura, e sem os dois juntos a linha fina está errada de um jeito ou
/// do outro:** o [`crate::binning::ScreenSpace::radius_px`] nunca deixa o raio abaixo de
/// `MIN_WIDTH_PX/2`, senão a fita não cobre o centro de nenhum pixel e a linha **pisca** ao mover ou
/// dar zoom (o rasterizador acerta ou erra o centro); mas desenhar 1,3 px onde o traço pede 0,3 põe
/// **quatro vezes** a tinta autorada. O fade devolve a energia: a **forma** fica no piso e a
/// **cobertura** desce. Sem ele o percurso desenhava toda linha sub-pixel grossa e opaca.
///
/// ⚠️ **Devolve exatamente `1.0` para espessura ≥ 1 px** (o `clamp` satura e `1·1·(3−2) = 1`), então
/// todo traço que o artista vê como uma linha normal fica byte-intocado.
///
/// ⚠️ **É a expansão do `smoothstep` da WGSL termo a termo** (`t = clamp((x−0)/(1−0), 0, 1)`, e
/// dividir por 1 é exato) — o `walk.wgsl` chama o **builtin**, para o percurso do device e o
/// rasterizador serem a MESMA aritmética; esta cópia é a do motor da CPU, com o gate irmão.
#[must_use]
pub fn sub_pixel_fade(w_px: f32) -> f32 {
    let t = w_px.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// **O QUE UM TRAÇO DEPOSITOU NESTE PIXEL** — `τ` e as propriedades médias da tinta que chegou.
///
/// ⚠️ **Durante a soma os campos derivados são SOMAS ponderadas por `dτ`; ao devolver eles são
/// MÉDIAS** — a divisão acontece uma vez, no fechamento do [`stroke_tau`]. É a resposta comutativa,
/// do mesmo tipo da lei.
///
/// ⚠️ **O `fade` viaja AQUI e não no [`StrokeStyle`], e a distinção é o desenho todo:** o estilo é o
/// que o traço DECLARA (a forma da queda, onde os dabs estão) e é lido uma vez; o fade é função do
/// **raio LOCAL**, então num traço que afina ele muda dab a dab. Um pixel é tocado por muitos dabs
/// de larguras diferentes, e o peso de cada um é exatamente o `dτ` que ele depositou — o MESMO peso
/// que a cor já usa.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Ink {
    /// A densidade acumulada. `α = 1 − exp(−τ)`.
    pub tau: f32,
    /// A cor média (o alfa já traz o `opacity`).
    pub rgba: [f32; 4],
    /// O [`sub_pixel_fade`] médio — o fator que a COBERTURA multiplica.
    pub fade: f32,
}

/// O que um traço deposita num pixel: `τ` acumulado, a cor e o fade médios (ponderados por `dτ`).
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
) -> Option<Ink> {
    let prof = style.profile;
    let tail = tail_point(data, run);
    // ⚠️ **Um acumulador, não três locais soltos** — durante a soma os campos derivados são somas
    // ponderadas por `dτ`; o `Ink` é o que os mantém viajando juntos, e é o que faz o `end_dab`
    // receber UM parâmetro em vez de três out-params que se pode esquecer de acrescentar.
    let mut ink = Ink {
        tau: 0.0,
        rgba: [0.0; 4],
        fade: 0.0,
    };
    for seg in run {
        let (pa, pb) = (data.points[seg.a as usize], data.points[seg.b as usize]);
        let sa = screen.point_px(pa.pos);
        let sb = screen.point_px(pb.pos);
        let ra = screen.radius_px(pa.width);
        let rb = screen.radius_px(pb.width);
        // As espessuras CRUAS deste segmento, e o atalho do caso comum. ⚠️ **Onde as duas pontas
        // medem ≥ 1 px toda amostra entre elas mede ≥ 1 px** (é uma combinação convexa), e ali o
        // [`sub_pixel_fade`] devolve `1.0` exato ⇒ o atalho pula trabalho sem mudar a resposta, e
        // um traço de espessura normal não paga um ciclo por esta wave.
        let wa = screen.thickness_px(pa.width);
        let wb = screen.thickness_px(pb.width);
        let full = wa >= 1.0 && wb >= 1.0;
        let fade_at = |f: f32| {
            if full {
                1.0
            } else {
                sub_pixel_fade(wa * (1.0 - f) + wb * f)
            }
        };
        let v = [sb[0] - sa[0], sb[1] - sa[1]];
        let len2 = v[0] * v[0] + v[1] * v[1];
        if len2 <= 1e-12 {
            continue;
        }
        let len = len2.sqrt();

        // A janela: os `t` onde a amostra pode estar dentro do disco de raio [`dab_reach`] em torno
        // de `p`. O `rmax` (e não o raio interpolado) mantém a janela CONSERVADORA — ela só pode
        // sobrar, nunca faltar — e o alcance sai da porta única porque um carimbo QUADRADO chega
        // mais longe que o raio dele. ⚠️ A janela vem da porta [`seg_window`], a MESMA que o
        // [`pass_end`] pergunta: se as duas divergissem, uma passagem entraria na soma sem entrar na
        // partição.
        let rmax = ra.max(rb);
        let reach = dab_reach(style.tip, rmax);
        let Some((t0, t1)) = seg_window(p, sa, sb, reach) else {
            continue;
        };

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
                ink.tau += d_tau;
                // A conta carrega UMA cor — a de onde ela está. É o que um carimbo é.
                let f = if wlen > 1e-12 {
                    ((k as f32 * pitch - arc_a) / wlen).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                ink.fade += fade_at(f) * d_tau;
                let op = pa.opacity * (1.0 - f) + pb.opacity * f;
                for (dst, (ca, cb)) in ink
                    .rgba
                    .iter_mut()
                    .zip(pa.color.iter().zip(&pb.color))
                    .take(3)
                {
                    *dst += (ca * (1.0 - f) + cb * f) * d_tau;
                }
                ink.rgba[3] += (pa.color[3] * (1.0 - f) + pb.color[3] * f) * op * d_tau;
            }
            continue;
        }

        // A grade da quadratura — do SEGMENTO, com o pitch mais apertado dele (conservador: mais
        // amostras, nunca menos).
        let pitch_min = (PAINTER_SPACING * 2.0 * ra.min(rb)).max(MIN_PITCH_PX);
        let ds = pitch_min / SUB as f32;
        // ⚠️ **A grade resolve a JANELA, não o SEGMENTO — e a densidade (`ds`) é a mesma.** O
        // integrando é zero fora de `[t0, t1]` por construção (a janela é exatamente onde o dab
        // alcança), então integrar a janela é a MESMA integral; o que muda é onde as amostras
        // pousam.
        //
        // Ancorada no SEGMENTO, um pixel cuja cobertura só vem de perto de um EXTREMO via o pico do
        // integrando cair em cima da fronteira do domínio: medido, suporte de **0,121 px contra um
        // passo de 0,35** ⇒ nenhuma amostra, `τ = 0`, e o percurso **derrubava a tinta** (doc 12
        // §22.8 — 4 px num traço reto, 13 num zigue-zague de 24 juntas). Ancorada na JANELA, o piso
        // de uma amostra cai no meio dela.
        //
        // ⚠️ **E o preço no miolo foi MEDIDO, não estimado** (§22.10): a mudança move **0 pixels
        // acima de 1/255** em `hardness` 0,4 e 0,7 (pior 0,10 e 0,37), porque ali a janela é larga
        // e re-ancorar a grade vale `O(passo²)`. Os 16 pixels que se movem em dureza 1 **são o
        // defeito sendo corrigido**.
        let win = (t1 - t0) * len;
        let n = (win / ds).ceil().max(1.0);
        let step = win / n;
        let i1 = n as u32 - 1;

        for i in 0..=i1 {
            let t = (t0 + (i as f32 + 0.5) * step / len).clamp(0.0, 1.0);
            let s = [sa[0] + v[0] * t, sa[1] + v[1] * t];
            let r = (ra * (1.0 - t) + rb * t).max(1e-4);
            let dn = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt() / r;
            let pitch = (PAINTER_SPACING * 2.0 * r).max(MIN_PITCH_PX);
            let d_tau = d_tau_of(dn, prof, step, r, pitch);
            if d_tau <= 0.0 {
                continue;
            }
            ink.tau += d_tau;
            ink.fade += fade_at(t) * d_tau;
            // A cor é a média ponderada por `dτ` — a resposta comutativa, do mesmo tipo da lei.
            // ⚠️ O `opacity` multiplica DEPOIS da cobertura (a regra do GP que o `flip.wgsl`
            // documenta: *um traço a opacity 0,5 não escurece sobre si mesmo*), então ele entra
            // no alfa da COR e nunca no `f`.
            let op = pa.opacity * (1.0 - t) + pb.opacity * t;
            for (dst, (ca, cb)) in ink
                .rgba
                .iter_mut()
                .zip(pa.color.iter().zip(&pb.color))
                .take(3)
            {
                *dst += (ca * (1.0 - t) + cb * t) * d_tau;
            }
            ink.rgba[3] += (pa.color[3] * (1.0 - t) + pb.color[3] * t) * op * d_tau;
        }
    }
    end_dab(run, data, screen, style, p, &mut ink);
    if ink.tau <= 0.0 {
        return None;
    }
    let tau = ink.tau;
    for c in &mut ink.rgba {
        *c /= tau;
    }
    ink.fade /= tau;
    Some(ink)
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
    ink: &mut Ink,
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
        ink.tau += d_tau;
        // Meio dab é um dab: ele carrega o fade da própria espessura, como qualquer outro.
        ink.fade += sub_pixel_fade(screen.thickness_px(pt.width)) * d_tau;
        for (dst, c) in ink.rgba.iter_mut().zip(&pt.color).take(3) {
            *dst += c * d_tau;
        }
        ink.rgba[3] += pt.color[3] * pt.opacity * d_tau;
    }
}
