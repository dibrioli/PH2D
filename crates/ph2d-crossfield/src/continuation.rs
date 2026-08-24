//! **A CONTINUAÇÃO DO ALINHAMENTO** — como o arredondamento inteiro do MIQ
//! absorve um termo que muda o `θ` debaixo dele.
//!
//! ⭐⭐ **O corte contra o [`super::solve`] é de ASSUNTO, e foi forçado pela HR-18**
//! (703 contra 700): lá mora **como se resolve uma vez** (o gauge da árvore, a
//! relaxação por CG, o arredondamento guloso); aqui **quantas vezes e com que
//! peso** — que é a pergunta que o relevo abriu.
//!
//! ⛔⛔ **CORREÇÃO (2026-08-24): a frase que estava aqui envelheceu, e ela mandava o
//! leitor para o diagnóstico errado.** Ela dizia *«hoje isto é código INERTE no
//! caminho que shipa ([`ALIGN_WEIGHT`] é `0`)»* — e o [`ALIGN_WEIGHT`] **shipa a
//! `0,03` desde 2026-08-22**. ⇒ **este código está VIVO nos dois caminhos do botão**
//! (o de sempre e o do mapa de grade inteira), e é ele que faz a grade obedecer ao
//! relevo. *O valor é o código; uma nota sobre ele é uma afirmação com data.*
//!
//! ⚠️ **O gate `the_continuation_is_inert_without_alignment` continua certo e continua
//! a valer** — mas ele fala do que acontece quando alguém **passa** `align = 0`, que é
//! a rede de recurso do produto, e **não** de qual é o peso que shipa.

use crate::{CrossField, Dual, QUARTER, wrap};

use super::solve::{Rounding, SolveReport, round_once};

/// ⭐⭐ **O PESO DO ALINHAMENTO na energia** — quanto a cruz é puxada para a
/// direção principal de curvatura, contra a suavidade.
///
/// ⛔ **Sem ele a energia é SÓ suavidade, e o campo não tem como ver o relevo.**
/// Medido em 2026-08-22 com a régua [`ph2d_quadfill::follows_relief`], na fixtura
/// com cristas: a cadeia dava **25,7°** de desvio — **pior que os 22,5° de uma
/// grade aleatória** — contra **13,7°** do porte do Instant Meshes.
///
/// ⚠️ **Ele é MULTIPLICADO PELA ANISOTROPIA de cada face**, então numa esfera
/// (onde as duas curvaturas são iguais) o termo encolhe-se sozinho. *Um alinhamento
/// que puxa onde a forma não tem direção põe uma costura onde ela não pede nenhuma.*
///
/// # ⭐⭐⭐ O NÚMERO SAIU DO GABARITO DO ORÁCULO, não de uma varredura no fim
///
/// ⛔ **A primeira varredura media o relevo da MALHA MONTADA** — onde o campo, o
/// traçado, a quantização e a montagem estão todos misturados. Ela dava tabelas
/// não-monótonas, escolhia `0,03` por acaso e o número partia noutro sítio; e a
/// conclusão de então foi *"o termo funciona mas não shipa"*.
///
/// ⭐⭐ **O oráculo grava o campo dele em disco** (`*_rem.rosy`, uma direção por
/// face — `PLAN.md` §4-duotricies). Medido **na malha remalhada DELE**, com a régua
/// da [`ph2d_quadfill::follows_relief`] aplicada ao **campo** e não à malha:
///
/// | peça | o NOSSO a `0` | ⭐ o do ORÁCULO | aleatório |
/// |---|---|---|---|
/// | com cristas | ⛔ **24,3°** | ⭐ **12,1°** | 22,5° |
/// | com bico | ⛔ **22,9°** | ⭐ **11,4°** | 22,5° |
/// | enrugada | 14,8° | 12,4° | 22,5° |
///
/// ⇒ **Duas das três ficavam do lado errado do aleatório.** Com um alvo à saída da
/// própria fase, o peso escolhe-se sozinho:
///
/// | peso | cristas (alvo 12,1°) | bico (11,4°) | enrugada (12,4°) | singularidades |
/// |---|---|---|---|---|
/// | `0` | ⛔ 24,3° | ⛔ 22,9° | 14,8° | 10 · 26 · 8 |
/// | `0,01` | 15,6° | 15,0° | 12,0° | 12 · 22 · 8 |
/// | ⭐⭐ **`0,03`** | **14,7°** | **13,9°** | **11,7°** | **12 · 22 · 8** |
/// | `0,1` | 13,2° | 12,5° | 11,4° | 12 · 29 · 8 |
/// | `1,0` | 9,7° | 10,5° | 9,9° | ⛔ 48 · 50 · 8 |
///
/// # ⭐⭐⭐ E a cadeia inteira, nas nossas fixturas
///
/// | fixtura | peso `0` | ⭐ **`0,03`** |
/// |---|---|---|
/// | ⛔ **orelha**, aresta máxima | ⛔ **56,5 % · 56,3 % · 57,0 %** da peça | ⭐ **12,4 % · 7,8 % · 5,5 %** |
/// | ⛔ **orelha**, dobras | ⛔ 42 · 134 · **2 204** | ⭐ **0 · 0 · 171** |
/// | com cristas, relevo | 24,2° | ⭐ **13,7°** |
/// | enrugada, relevo | 23,1° | ⭐ **13,8°** |
/// | com bico, relevo | 23,9° | ⭐ **19,0°** |
///
/// ⭐⭐ **É a aresta que o artista fotografou três vezes**, e ela desaparece.
///
/// ⚠️ **`0,1` é melhor no campo e recusa na cadeia** (a quantização do gancho não
/// fecha). `0,03` é o maior peso que fecha em todas as cinco fixturas.
///
/// # ⚠️ O preço, dito por extenso
///
/// ⛔ **O toro 32×16 passa a RECUSAR** (`GenusLost`) onde a `0` ele entregava malha.
/// ⭐ **E a rede apanha-o:** a porta do produto tenta o campo alinhado e cai para o
/// só-suavidade quando o layout não fecha (`quad_remesh_global`), com o relatório a
/// dizer qual correu. *A rede foi construída no mesmo dia e é agora que ela ganha o
/// lugar dela.*
pub const ALIGN_WEIGHT: f32 = 0.03;

/// ⭐⭐ **A CONTINUAÇÃO DO ALINHAMENTO** — como o arredondamento inteiro absorve
/// um termo que muda o `θ` debaixo dele.
///
/// # ⛔ O mecanismo que ela cura, escrito por extenso
///
/// O arredondamento guloso congela `livres/8` inteiros **depois da primeira
/// resolução** e nunca os revisita. Sem alinhamento isso é benigno: o `θ` da
/// primeira resolução já é o campo suave, e as resoluções seguintes só o afinam.
///
/// ⚠️ **Com alinhamento a primeira resolução parte de `θ = 0`** — e o
/// representante 4-RoSy de cada face é escolhido *pelo `θ` corrente*
/// ([`solve_relaxation`]). A `θ = 0`, o alvo de toda face é `α_f` dobrado para
/// `[−π/4, π/4]`, que **não é** o braço que o campo convergido quer. O guloso
/// congela um oitavo dos inteiros sobre esse alvo errado, e cada inteiro errado é
/// uma singularidade a mais. *Não é o termo que parte o traçado: é decidir sobre
/// um `θ` que ainda vai mudar.*
///
/// # As três curas, e as três são da mesma família do F4
///
/// | campo | o que faz |
/// |---|---|
/// | [`Self::warm`] | corre **uma passagem inteira com `align = 0`** só para semear o `θ`. O representante 4-RoSy passa a ser escolhido sobre um campo cruzado válido, e não sobre zeros |
/// | [`Self::recentres`] | **refaz o arredondamento do zero**, semeado pelo `θ` da passagem anterior, enquanto o objectivo melhorar |
/// | ⭐ [`Self::ramp`] | reparte o peso pelas passagens (`w/N`, `2w/N`, …, `w`) — cada passagem vê uma **perturbação pequena** do campo que a anterior já resolveu |
///
/// ⭐ É *aproximar, re-centrar, repetir* — a mesma lei que o
/// [`ph2d_quantize::quantize_within`] recebeu, e pela mesma razão: **a referência
/// não congela guloso sobre um alvo que ela própria acabou de mover.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Continuation {
    /// Quantas vezes o arredondamento inteiro é **refeito** — cada uma semeada
    /// pelo `θ` da anterior, com **todos** os inteiros livres outra vez. O laço
    /// pára assim que uma passagem não melhora o objectivo.
    pub recentres: usize,
    /// Se uma passagem de **aquecimento** com `align = 0` corre antes, só para
    /// semear o `θ`.
    pub warm: bool,
    /// ⭐ **QUANTOS DEGRAUS a escada de pesos tem antes do peso cheio.** Com `3`, as
    /// passagens correm a `w/4`, `w/2`, `3w/4` e só então a `w`.
    ///
    /// ⚠️ **É o sentido literal de «continuação»**, e ataca o mecanismo por outro
    /// lado que o [`Self::warm`]: aquele dá um bom ponto de partida **uma vez**;
    /// esta garante que **nenhuma** passagem vê um salto grande no alvo. *Se o
    /// guloso parte por causa do tamanho da perturbação, é esta que o cura.*
    ///
    /// ⚠️ **Um degrau NÃO é candidato a resultado** — ele resolve outro problema
    /// (um alinhamento mais fraco), e deixá-lo ganhar seria entregar um campo que
    /// ninguém pediu. Os degraus só semeiam; quem compete são as passagens de peso
    /// cheio, e o objectivo delas é medido **no peso final**.
    pub ramp_steps: usize,
}

impl Default for Continuation {
    /// ⭐⭐ **A SEMENTE SOZINHA, e os outros dois a ZERO — por medição.**
    ///
    /// Varredura de 2026-08-22 (`does_the_continuation_save_the_alignment_term`),
    /// fixtura **com cristas**, peso `0,01`. A base (`peso 0`) é
    /// *21 patches · valência 5 · 14 irregulares · 0 dobras · 25,7°*:
    ///
    /// | lei | resoluções | patches | valência | irreg | dobras | ⭐ relevo |
    /// |---|---|---|---|---|---|---|
    /// | `cru` (a lei antiga) | 56 | ⛔ **104** | ⛔ 15 | ⛔ 139 | ⛔ 23 | 20,4° |
    /// | ⭐ **`warm`** | 112 | **17** | **4** | **10** | **0** | ⭐ **16,7°** |
    /// | `warm` + 4 re-centragens | 168 | 17 | 4 | 10 | 0 | 16,7° |
    /// | `rampa3` sem semente | 224 | 18 | 7 | 26 | 2 | 16,6° |
    /// | `warm` + `rampa3` | 280 | 17 | 5 | 14 | 3 | 17,5° |
    /// | `warm`+`rampa3`+re4 | 448 | 6 | 4 | 11 | 0 | ⛔ 24,2° |
    ///
    /// ⭐ **A semente sozinha ganha em TODAS as colunas** — e ganha à própria base
    /// sem alinhamento: menos patches (17 contra 21), menos irregulares (10 contra
    /// 14), zero dobras, e o relevo de 25,7° para 16,7°. *A explosão nunca foi do
    /// termo: era do `θ = 0`.*
    ///
    /// ⛔ **As outras duas leis foram MEDIDAS E REJEITADAS**, e não por serem
    /// caras: a última linha custa **8× as resoluções** e devolve **24,2°** —
    /// praticamente uma grade que não olhou. O mecanismo está identificado: o
    /// árbitro entre passagens é o [`objective`], onde a `0,01` a **suavidade
    /// domina** o termo de alinhamento; re-centrar caminha então para o campo mais
    /// liso, que é exatamente aquele que ignora o relevo. *Um laço que melhora o
    /// número que mede não melhora o número que importa, quando são dois números.*
    ///
    /// ⚠️ **Ficam como parâmetros, não como código morto:** são eles que tornam
    /// esta tabela reproduzível, e a rejeição é do *default*, não da existência.
    fn default() -> Self {
        Self {
            recentres: 0,
            warm: true,
            ramp_steps: 0,
        }
    }
}

/// **O MIQ com o PESO DO ALINHAMENTO explícito** — ver [`ALIGN_WEIGHT`].
///
/// ⚠️ **Ele é um parâmetro porque o número tem de sair de uma VARREDURA**, e a
/// primeira tentativa provou-o: com `1,0` a cadeia inteira parou de fechar —
/// patches de **39, 98 e 127 lados**, quantização a recusar em toda fixtura.
/// *Um peso de energia escolhido sem tabela é um palpite com aparência de teoria.*
#[must_use]
pub fn solve_miq_aligned(dual: &Dual, policy: Rounding, align: f32) -> (CrossField, SolveReport) {
    solve_miq_continued(dual, policy, align, Continuation::default())
}

/// ⭐⭐ **O MIQ com a CONTINUAÇÃO na mão** — ver [`Continuation`].
///
/// ⚠️ **Ponto de extensão append-only** (`CLAUDE.md` §0.2): com
/// `Continuation { recentres: 0, warm: false }` ele é byte-idêntico à lei antiga,
/// e há gate a exigi-lo (`the_continuation_is_inert_without_alignment`).
#[must_use]
pub fn solve_miq_continued(
    dual: &Dual,
    policy: Rounding,
    align: f32,
    cont: Continuation,
) -> (CrossField, SolveReport) {
    let n = dual.frames().len();
    let mut report = SolveReport {
        solves: 0,
        free_integers: 0,
        nonzero_periods: 0,
        cg_capped: 0,
        cg_worst_residual: 0.0,
        recentres: 0,
    };
    if n == 0 {
        return (
            CrossField {
                theta: Vec::new(),
                period: Vec::new(),
            },
            report,
        );
    }

    let schedule = weight_schedule(align, cont);
    let mut theta = vec![0.0f32; n];
    let mut best: Option<(f64, Vec<f32>, Vec<i32>)> = None;
    let mut kept = 0usize;

    for &w in &schedule {
        let mut next_theta = theta.clone();
        let next_period = round_once(dual, policy, w, &mut next_theta, &mut report);
        // ⚠️ **A semente da passagem seguinte é sempre a ÚLTIMA, mesmo que ela
        // tenha sido pior.** Semear com a melhor faria o laço reproduzir a mesma
        // passagem para sempre; o que se guarda por qualidade é o *resultado*, não
        // o ponto de partida.
        theta = next_theta.clone();
        // Uma passagem da escada não é candidata: ela resolve outro problema.
        if w + 1.0e-9 < align {
            continue;
        }
        let e = objective(dual, &next_theta, &next_period, align);
        match &best {
            Some((prev, ..)) if e >= *prev - 1.0e-9 => break,
            _ => {}
        }
        if best.is_some() {
            kept += 1;
        }
        best = Some((e, next_theta, next_period));
    }

    // ⚠️ O `expect` é inalcançável: [`weight_schedule`] garante ≥1 passagem com o
    // peso final, e ela é sempre candidata.
    let (_, theta, period) = best.expect("a escada de pesos tem de conter o peso final");
    report.recentres = kept;
    report.nonzero_periods = period.iter().filter(|p| **p != 0).count();
    (CrossField { theta, period }, report)
}

/// **A ESCADA DE PESOS** — uma entrada por passagem do arredondamento.
///
/// ⚠️ **Com `align = 0` ela devolve exactamente UMA passagem**, e é isso que torna
/// a [`Continuation`] inerte no caminho que shipa. Sem esta guarda, um
/// `recentres: 4` correria cinco arredondamentos idênticos para chegar ao mesmo
/// campo — cinco vezes o relógio por uma feature desligada.
fn weight_schedule(align: f32, cont: Continuation) -> Vec<f32> {
    if align <= 0.0 {
        return vec![0.0];
    }
    let mut out = Vec::with_capacity(cont.ramp_steps + cont.recentres + 2);
    if cont.warm {
        out.push(0.0);
    }
    for k in 1..=cont.ramp_steps {
        #[allow(clippy::cast_precision_loss)]
        let step = align * k as f32 / (cont.ramp_steps + 1) as f32;
        out.push(step);
    }
    // ⭐ O peso cheio, uma vez mais uma por re-centragem pedida.
    out.extend(std::iter::repeat_n(align, cont.recentres + 1));
    out
}

/// **O OBJECTIVO COMPLETO** — a suavidade **mais** o alinhamento.
///
/// ⚠️ **Comparar só a suavidade seria escolher a rodada errada.** Uma passagem
/// re-centrada pode gastar suavidade de propósito para pagar alinhamento; a régua
/// que decide qual passagem fica tem de ser a mesma que o solver minimiza, senão a
/// [`Continuation`] pára justamente quando o termo começou a funcionar.
///
/// ⚠️ **O desvio ao alvo é dobrado para `[−π/4, π/4]`** — a cruz tem quatro
/// braços, e medir o desvio cru daria uma energia que depende de qual deles o
/// `solve_relaxation` escolheu representar.
fn objective(dual: &Dual, theta: &[f32], period: &[i32], align: f32) -> f64 {
    let mut sum = 0.0f64;
    for (e, de) in dual.edges().iter().enumerate() {
        let r = wrap(
            theta[de.f as usize] - theta[de.g as usize] + de.kappa + QUARTER * period[e] as f32,
        );
        sum += f64::from(de.weight) * f64::from(r) * f64::from(r);
    }
    if align <= 0.0 {
        return sum;
    }
    for (f, t) in theta.iter().enumerate() {
        let (a, conf) = dual.align(f);
        if conf <= 0.0 {
            continue;
        }
        let d = t - a - QUARTER * ((t - a) / QUARTER).round();
        sum += f64::from(align) * f64::from(conf) * f64::from(d) * f64::from(d);
    }
    sum
}
