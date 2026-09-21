//! DIAGNÓSTICO — o PREÇO de estender o **Composite Brush** de TRÊS para CINCO camadas.
//!
//! Ordem do dono (2026-09-20): *«Em Digital:Composite Brush temos 3 tools juntas Brush+Smear+Blur.
//! Meça custo de colocar mais 2 tool nesse cadeia: Erase + um segundo Brush. Nos brushes entram
//! campos de cor e tamanho do carimbo além do Strength que já tem. Smear e Blur ganham o tamanho do
//! carimbo além do Strength que já tem.»*
//!
//! ⚠️ **Isto NÃO implementa nada.** É a medição que a decisão exige: o preço MARGINAL de uma camada
//! dentro da pilha REAL, medido pela porta do produto ([`PainterTool::on_canvas_pointer`]), mais a
//! lei do TAMANHO (o que um carimbo maior cobra por camada) e a razão borracha/pincel.
//!
//! ## Por que MARGINAL e não um número absoluto
//!
//! A pilha não é uma soma de operações independentes: quando há uma camada **Smear** viva, toda
//! camada que NÃO é smear paga o depósito **DUAS vezes** — uma no canvas e outra na base congelada
//! ⚠️⚠️ **A DOBRA que este cabeçalho mede MORREU em 2026-09-20** (a wave da ordem por TRAÇO): a
//! base do knife passou a ser **refrescada** da tela no momento em que a vez dele chega na pilha,
//! em vez de toda camada depositar duas vezes. As tabelas abaixo ficam como a medição do preço que
//! ela custava — *`×2,09` num Brush e `×1,92` num Blur* —, e são elas que dizem o que a cura poupou.
//!
//! da sessão de esfregar (a DOBRA, `lay_into_smear_base` — a cura de
//! 2026-08-09 sem a qual o traço perde 33 das 141 colunas). ⇒ *o preço de uma camada depende de
//! quem mais está na pilha*, e um número medido sozinho subestima.
//!
//! ## As colunas
//!
//! Cada linha é um traço inteiro (down → N moves → up) num canvas de `1024²`, cronometrado pelo
//! MÍNIMO de cinco corridas (a mediana de um relógio de parede desta máquina é ruído; o mínimo é o
//! que a máquina de facto consegue). O `loadavg` sai ao lado — ⚠️ **nenhuma leitura destas vale
//! nada acima de `load ~5`** (CLAUDE.md §5.0).
//!
//! ## O que a extensão para cinco camadas NÃO custa, e está medido aqui
//!
//! A **cor por camada** é um campo que o [`ph2d_painter_brush::Dab`] já carrega (`Dab::color`), e o
//! **tamanho** também (`Dab::radius_px`). ⛔ Mas o tamanho **não é um multiplicador aplicável ao
//! dab**: o ESPAÇAMENTO entre dabs foi resolvido pelo motor de traço com o raio do pincel
//! (`spacing × diâmetro`), logo uma camada mais FINA sobre a mesma lista de dabs sai em CONTAS e uma
//! mais GROSSA paga sobreposição a mais (o endurecimento que o doc 25 §13.10 já mede). Um tamanho
//! por camada honesto é **um percurso de traço por camada** — `[Stroke; N]` em vez de `Stroke` —, e
//! é isso que a linha `dabs` da tabela do tamanho mostra: a contagem de dabs muda com o raio.

use super::*;
use std::time::Instant;

/// Canvas do tamanho de trabalho — um `200²` de fixtura mediria o overhead e não o depósito.
const SIZE: u32 = 1024;
/// O raio de referência das tabelas (o pincel de trabalho do dono na foto do composite era `34`).
const RAIO: f32 = 24.0;
/// Corridas por célula; fica-se com o MÍNIMO.
const CORRIDAS: usize = 5;
/// Pares por marginal — a mediana das diferenças EMPARELHADAS ([`marginal`]).
const CORRIDAS_PAR: usize = 15;

fn ponteiro(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço reto de `720 px`, entregue em passos de `2 px` — o que um rato de alta taxa produz.
fn traco(t: &mut PainterTool) {
    let (y, x0, x1) = (512.0f32, 150.0f32, 870.0f32);
    t.on_canvas_pointer(ponteiro([x0, y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 2.0;
        t.on_canvas_pointer(ponteiro([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(ponteiro([x1, y], PointerPhase::Up));
}

/// Uma ferramenta com o pincel do PRODUTO (não a fixtura de disco duro dos gates) sobre papel branco.
fn ferramenta(raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = raio;
    t.paint.brush.color = [0.6, 0.0, 0.0];
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t
}

/// Arma a pilha: `ops[i]` ocupa a posição `i` (0 = topo, corre por ÚLTIMO). As posições que sobram
/// levam Strength `0`, que é como o motor pula uma camada.
fn pilha_com(t: &mut PainterTool, ops: &[(CompositeOp, f32)]) {
    pilha(t, ops);
}

fn pilha(t: &mut PainterTool, ops: &[(CompositeOp, f32)]) {
    t.paint.composite_enabled = !ops.is_empty();
    for pos in 0..3usize {
        match ops.get(pos) {
            Some(&(op, s)) => {
                t.paint.composite[pos] = CompositeLayer {
                    op,
                    strength: s,
                    ..CompositeLayer::default()
                }
            }
            None => t.paint.composite[pos].strength = 0.0,
        }
    }
}

/// Uma corrida: o relógio de um traço inteiro, em ms. A tela é alocada FORA da janela.
fn corrida(raio: f32, ops: &[(CompositeOp, f32)], borracha: bool) -> f64 {
    let mut t = ferramenta(raio);
    t.paint.eraser = borracha;
    pilha(&mut t, ops);
    let t0 = Instant::now();
    traco(&mut t);
    t0.elapsed().as_secs_f64() * 1e3
}

/// O relógio de um traço inteiro, em ms — mínimo de [`CORRIDAS`].
fn ms(raio: f32, ops: &[(CompositeOp, f32)], borracha: bool) -> f64 {
    (0..CORRIDAS).fold(f64::MAX, |m, _| m.min(corrida(raio, ops, borracha)))
}

/// ⚠️⚠️ **O MARGINAL de uma camada mede-se EMPARELHADO, nunca subtraindo dois mínimos.**
///
/// A 1.ª redacção desta sonda fazia `min(B) − min(A)` sobre corridas separadas, e numa das leituras
/// isso deu **`−0,54 ms`** para uma camada que custa `~9`: a dispersão do Smear entre corridas desta
/// máquina é de **`14 ms`** (leu `67,56` e `81,60` no mesmo dia), e uma diferença de dois números
/// ruidosos não é uma marginal — é a deriva da máquina com o sinal do acaso.
///
/// ⇒ aqui `A` e `B` correm **lado a lado dentro da mesma iteração**, e o que se devolve é a
/// **MEDIANA das diferenças emparelhadas**: a deriva lenta entra nos dois lados da mesma subtracção
/// e cancela-se. A dispersão sai junto (`p10`..`p90`) — *uma marginal sem a dispersão ao lado não
/// diz se ela é mensurável*.
fn marginal(raio: f32, a: &[(CompositeOp, f32)], b: &[(CompositeOp, f32)]) -> (f64, f64, f64) {
    let mut d: Vec<f64> = (0..CORRIDAS_PAR)
        .map(|_| {
            let ta = corrida(raio, a, false);
            let tb = corrida(raio, b, false);
            tb - ta
        })
        .collect();
    d.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let q = |f: f64| d[((d.len() - 1) as f64 * f).round() as usize];
    (q(0.5), q(0.1), q(0.9))
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

use CompositeOp::{Blur as BL, Brush as B, Smear as S};

/// Eventos de ponteiro que um traço entrega (1 Down + N Moves + 1 Up) — o divisor do «por evento».
const EVENTOS: f64 = 362.0;

/// SONDA — o preço MARGINAL de cada camada dentro da pilha viva, e a previsão das CINCO.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_o_preco_marginal_de_uma_camada() {
    eprintln!(
        "loadavg {} · canvas {SIZE}² · raio {RAIO} · traço 720 px ({EVENTOS:.0} eventos)",
        carga()
    );
    let c0 = ms(RAIO, &[], false);
    let c_s = ms(RAIO, &[(S, 0.5)], false);
    let c_b = ms(RAIO, &[(B, 1.0)], false);
    let c_bl = ms(RAIO, &[(BL, 0.5)], false);
    let c_bs = ms(RAIO, &[(B, 1.0), (S, 0.5)], false);
    let c_bbs = ms(RAIO, &[(B, 1.0), (B, 1.0), (S, 0.5)], false);
    let c_bsbl = ms(RAIO, &[(B, 1.0), (S, 0.5), (BL, 0.5)], false);

    let linha =
        |nome: &str, v: f64| eprintln!("  {nome:<34} {v:7.2} ms   {:6.3} ms/evento", v / EVENTOS);
    linha("pincel sozinho (pilha DESLIGADA)", c0);
    linha("[Brush]", c_b);
    linha("[Smear]", c_s);
    linha("[Blur]", c_bl);
    linha("[Brush+Smear]", c_bs);
    linha("[Brush+Brush+Smear]", c_bbs);
    linha("[Brush+Smear+Blur]  <- HOJE", c_bsbl);

    // ⚠️ **A ADITIVIDADE tem de ser medida contra as partes INDEPENDENTES.** A 1.ª redacção desta
    // sonda somava as marginais (`c_s + (c_bs−c_s) + (c_bsbl−c_bs)`), que TELESCOPA para `c_bsbl`
    // por construção e imprimia `+0,0 %` — um controlo vazio, verde a afirmar nada.
    let soma_independente = c_b + c_s + c_bl;
    eprintln!(
        "  --- as partes SOZINHAS somam {soma_independente:.2} e a pilha mede {c_bsbl:.2} ⇒ a pilha cobra {:+.1} % A MAIS",
        100.0 * (c_bsbl - soma_independente) / soma_independente
    );

    // ⚠️ EMPARELHADAS — ver o doc de [`marginal`]: `min(B) − min(A)` já leu `−0,54 ms` aqui.
    let (m_b1, b1lo, b1hi) = marginal(RAIO, &[(S, 0.5)], &[(B, 1.0), (S, 0.5)]);
    let (m_b2, b2lo, b2hi) = marginal(RAIO, &[(B, 1.0), (S, 0.5)], &[(B, 1.0), (B, 1.0), (S, 0.5)]);
    let (m_bl, bllo, blhi) = marginal(
        RAIO,
        &[(B, 1.0), (S, 0.5)],
        &[(B, 1.0), (S, 0.5), (BL, 0.5)],
    );
    eprintln!(
        "  --- marginais EMPARELHADAS, dentro da pilha (a base de smear viva ⇒ a DOBRA arma) ---"
    );
    eprintln!(
        "  1.o Brush   {m_b1:6.2} ms [p10 {b1lo:.2} .. p90 {b1hi:.2}]   contra {c_b:.2} sozinho ⇒ dobra x{:.2}",
        m_b1 / c_b
    );
    eprintln!(
        "  2.o Brush   {m_b2:6.2} ms [p10 {b2lo:.2} .. p90 {b2hi:.2}]   <- a camada que o dono quer acrescentar"
    );
    eprintln!(
        "  Blur        {m_bl:6.2} ms [p10 {bllo:.2} .. p90 {blhi:.2}]   contra {c_bl:.2} sozinho ⇒ dobra x{:.2}",
        m_bl / c_bl
    );

    // A 5.ª camada é um Erase: mesma rota de depósito, outro blend (ver a sonda da borracha).
    let razao_borracha = {
        let p = ms(RAIO, &[], false);
        let e = ms(RAIO, &[], true);
        e / p
    };
    let previsto = c_bsbl + m_b2 + m_b2 * razao_borracha;
    eprintln!(
        "  --- PREVISTO [Brush+Brush+Erase+Smear+Blur] = {previsto:.2} ms ({:6.3} ms/evento) = x{:.2} a pilha de hoje",
        previsto / EVENTOS,
        previsto / c_bsbl
    );
}

/// SONDA — **um SEGUNDO Brush na pilha chega a depositar?** (a pergunta de VIABILIDADE, não de preço)
///
/// O cap de Accumulate é um `stroke_mask` **por TRAÇO**, e ele arma quando
/// `!accumulate && strength < 1.0` ([`super::stamp_route::PainterTool::stroke_cover_wanted`]). Duas
/// camadas Brush no mesmo traço partilham esse mapa ⇒ a hipótese é que o 1.º pincel leva a
/// cobertura ao tecto e o 2.º **deposita ZERO** — exactamente o defeito que o
/// `lay_into_smear_base` já teve de contornar salvando e repondo o `stroke_mask` à volta da dobra.
///
/// ⚠️ A régua é a TINTA depositada (quanto o papel escureceu ao longo do traço), e ela é lida com
/// o CONTROLO ao lado: a MESMA pilha com uma camada só. Sem ele, um número sozinho não diz se a
/// segunda camada trabalhou.
///
/// ## MEDIDO (2026-09-20) — o 2.º Brush acrescenta `+0,0 %` em TODA a faixa de Strength
///
/// | Strength | UM Brush | DOIS | o 2.º acrescentou |
/// |---|---|---|---|
/// | 1,0 | 101,99 | 102,00 | **+0,01** |
/// | 0,6 |  61,08 |  61,04 | **−0,04** |
/// | 0,3 |  30,73 |  30,67 | **−0,05** |
///
/// ⭐ **E a aritmética separa DUAS causas, que pedem curas diferentes:**
/// - **Em `1,0` o cap está DESARMADO** (`stroke_cover_wanted` é `strength < 1.0`) — os dois passes
///   depositam, e o segundo é invisível porque pinta a MESMA cor por cima de tinta opaca. ⇒ isto
///   cura-se com a **cor por camada**, que é o que o dono pediu.
/// - **Abaixo de `1,0` quem bloqueia é o CAP, e a cor não o cura.** Pintar `0,6` sobre branco com
///   a cor `[0,6, 0, 0]` escurece o canal vermelho `0,6 × 0,4 × 255 = 61,2` — e o medido é
///   `61,04`. Se o 2.º passe depositasse, a cobertura combinada seria `1 − 0,4² = 0,84` ⇒ `~86`.
///   *O número lido é EXACTAMENTE o cap, ao décimo.* ⇒ a cura é um `stroke_mask` **por camada**,
///   que é a mesma coisa que a DOBRA (`lay_into_smear_base`) já tinha de
///   fazer à mão (ele salva e repõe o mapa à volta da dobra, e o doc dele diz porquê).
#[test]
#[ignore = "diagnóstico: roda sob demanda"]
fn diag_um_segundo_brush_na_pilha_chega_a_depositar() {
    eprintln!("A TINTA que a pilha deixa (media do escurecimento na linha do traço, 0..255):");
    for strength in [1.0f32, 0.6, 0.3] {
        let tinta = |ops: &[(CompositeOp, f32)]| -> f64 {
            let mut t = ferramenta(RAIO);
            pilha(&mut t, ops);
            traco(&mut t);
            let y = 512u32;
            let n = 870 - 150;
            (150u32..870)
                .map(|x| {
                    let i = ((y * SIZE + x) * 4) as usize;
                    f64::from(255 - t.canvas_rgba[i])
                })
                .sum::<f64>()
                / f64::from(n)
        };
        let um = tinta(&[(B, strength), (S, 0.5)]);
        let dois = tinta(&[(B, strength), (B, strength), (S, 0.5)]);
        eprintln!(
            "  Strength {strength:.1}  ·  UM Brush {um:6.2}  ·  DOIS {dois:6.2}  ⇒ o 2.o acrescentou {:+.2} ({:+.1} %)",
            dois - um,
            100.0 * (dois - um) / um.max(1e-9)
        );
    }
}

/// SONDA — **o preço de REUSAR a lista de dabs num carimbo maior.**
///
/// Um tamanho por camada tem duas implementações possíveis, e esta sonda mede a diferença SEM tocar
/// no produto, por uma equivalência exacta: o motor emite dabs a `spacing × diâmetro` de distância,
/// logo a lista de um pincel de raio `r` reutilizada por uma camada de raio `k·r` é, para essa
/// camada, **a mesma lista que ela emitiria com `spacing/k`**.
///
/// ⇒ `raio 96, spacing 0,025` é EXACTAMENTE «a camada de raio 96 a reutilizar a lista de um pincel
/// de raio 24 a `spacing 0,10`» (o valor de fábrica), e `raio 96, spacing 0,10` é «a camada com o
/// percurso dela própria». A diferença entre as duas colunas é o preço de reutilizar a lista.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_o_preco_de_reusar_a_lista_de_dabs_num_carimbo_maior() {
    eprintln!("loadavg {} · canvas {SIZE}² · traço 720 px", carga());
    eprintln!("  A camada GRANDE (raio 96) sobre a lista de um pincel de raio 24:");
    eprintln!("   op    | percurso PROPRIO (sp 0,100) | lista REUSADA (sp 0,025) |  razao");
    for (nome, op) in [("Brush", B), ("Smear", S), ("Blur ", BL)] {
        let medir = |sp: f32| -> f64 {
            let mut melhor = f64::MAX;
            for _ in 0..CORRIDAS {
                let mut t = ferramenta(96.0);
                t.paint.brush.spacing = sp;
                pilha(&mut t, &[(op, if matches!(op, B) { 1.0 } else { 0.5 })]);
                let t0 = Instant::now();
                traco(&mut t);
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            melhor
        };
        let proprio = medir(0.100);
        let reusada = medir(0.025);
        eprintln!(
            "   {nome} | {proprio:22.2} ms | {reusada:19.2} ms |  x{:.2}",
            reusada / proprio
        );
    }
}

/// SONDA — o custo segue a TELA ou o TRAÇO? (decide se cinco camadas escalam com o documento)
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_o_custo_segue_a_tela_ou_o_traco() {
    eprintln!("loadavg {} · raio {RAIO} · traço 720 px", carga());
    eprintln!("  tela | [Brush] | [Smear] |  [Blur] | pilha de HOJE");
    for lado in [512u32, 1024, 2048] {
        let medir = |ops: &[(CompositeOp, f32)]| -> f64 {
            let mut melhor = f64::MAX;
            for _ in 0..CORRIDAS {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
                t.paint.brush.radius_px = RAIO;
                t.paint.brush.color = [0.6, 0.0, 0.0];
                t.paint.brush.strength = 1.0;
                t.paint.brush.space_attenuation = false;
                pilha(&mut t, ops);
                let y = lado as f32 * 0.5;
                let (x0, x1) = (lado as f32 * 0.15, lado as f32 * 0.85);
                let t0 = Instant::now();
                t.on_canvas_pointer(ponteiro([x0, y], PointerPhase::Down));
                let mut x = x0;
                while x < x1 {
                    x += 2.0;
                    t.on_canvas_pointer(ponteiro([x, y], PointerPhase::Move));
                }
                t.on_canvas_pointer(ponteiro([x1, y], PointerPhase::Up));
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            melhor
        };
        let b = medir(&[(B, 1.0)]);
        let s = medir(&[(S, 0.5)]);
        let bl = medir(&[(BL, 0.5)]);
        let hoje = medir(&[(B, 1.0), (S, 0.5), (BL, 0.5)]);
        eprintln!("  {lado:4} | {b:7.2} | {s:7.2} | {bl:7.2} | {hoje:7.2}");
    }
}

/// SONDA — a BORRACHA custa o que o pincel custa? (a 5.ª camada é um `Erase`, e a pilha de hoje
/// RECUSA o modo borracha — `composite_active()` exige `!eraser` —, logo ela mede-se de fora.)
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_a_borracha_custa_o_que_o_pincel_custa() {
    eprintln!("loadavg {}", carga());
    let pincel = ms(RAIO, &[], false);
    let borracha = ms(RAIO, &[], true);
    eprintln!("  pincel   {pincel:7.2} ms");
    eprintln!(
        "  borracha {borracha:7.2} ms   ⇒ razão ×{:.3}",
        borracha / pincel
    );
}

/// SONDA — a lei do TAMANHO: o que uma camada cobra quando o carimbo dela é maior.
///
/// ⚠️ A contagem de dabs sai junto porque ela é metade da resposta: o custo de uma passagem é
/// `dabs × área do dab`, e `dabs ∝ 1/raio` (o espaçamento é uma fracção do DIÂMETRO) ⇒ o custo por
/// comprimento de traço deve ser ~LINEAR no raio, não quadrático.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_o_preco_do_tamanho_por_camada() {
    eprintln!("loadavg {} · canvas {SIZE}² · traço 720 px", carga());
    eprintln!("  raio |   Brush |   Smear |    Blur");
    for raio in [12.0f32, 24.0, 48.0, 96.0] {
        let b = ms(raio, &[(B, 1.0)], false);
        let s = ms(raio, &[(S, 0.5)], false);
        let bl = ms(raio, &[(BL, 0.5)], false);
        eprintln!("  {raio:4.0} | {b:7.2} | {s:7.2} | {bl:7.2}");
    }
}

/// SONDA — **o NÚCLEO DE CAIXA no Blur da pilha: quanto mais leve, e quanto mais diferente.**
///
/// Ordem do dono (2026-09-20): *«veja se abaixando a qualidade do blur não fica bem mais leve. Mas
/// só no Blur do composite. O Blur como ferramenta isolada não deve ser modificado.»*
///
/// ⭐ **As duas colunas são as duas ROTAS DO PRODUTO, não dois arneses:** o binomial é a ferramenta
/// Blur isolada (`PaintMode::Blur`, pilha desligada) e a caixa é a pilha com uma camada Blur só —
/// que, sem Smear na pilha, não paga a dobra. *Medir o núcleo por uma porta de teste mediria uma
/// função; medi-lo assim mede o que o artista corre.*
///
/// As DUAS colunas juntas, porque cada uma sozinha mente: um núcleo mais barato que borre outra
/// coisa não é «a mesma qualidade mais leve», e uma diferença de imagem sem o relógio ao lado não
/// diz se valeu a pena. A régua da imagem é o **pior byte** entre as duas saídas e a **média**.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, em --release e com a máquina calma"]
fn diag_o_nucleo_de_caixa_contra_o_binomial() {
    eprintln!("loadavg {} · canvas {SIZE}² · traço 720 px", carga());
    eprintln!("  raio | binomial |   caixa | ganho p50 (p10..p90) | pior byte |  media");
    for raio in [12.0f32, 24.0, 48.0, 96.0] {
        // Uma rota: pinta uma marca (para o blur ter o que borrar) e depois passa o blur por cima.
        let corre = |pilha: bool| -> (f64, Vec<u8>) {
            let mut t = ferramenta(raio);
            t.paint.composite_enabled = false;
            traco(&mut t); // a marca, com o pincel normal
            if pilha {
                t.paint.composite_enabled = true;
                pilha_com(&mut t, &[(BL, 1.0)]);
            } else {
                t.paint.paint_mode = PaintMode::Blur;
            }
            let t0 = Instant::now();
            traco(&mut t);
            (t0.elapsed().as_secs_f64() * 1e3, t.canvas_rgba.to_vec())
        };
        // ⚠️ **PAREADO:** os dois lados na MESMA iteração, e a estatística é sobre a RAZÃO de cada
        // par. Tomar `min(B)/min(A)` de corridas separadas é a forma que esta sessão já pagou uma
        // vez (um marginal de ~10 ms lido como `−0,54`): com uma vizinha a 1110 % de CPU, dois
        // mínimos de janelas diferentes não são comparáveis.
        let mut razoes = Vec::with_capacity(CORRIDAS_PAR);
        let (mut tb0, mut tc0) = (f64::MAX, f64::MAX);
        let (mut ib, mut ic) = (Vec::new(), Vec::new());
        for _ in 0..CORRIDAS_PAR {
            let (tb, b) = corre(false);
            let (tc, c) = corre(true);
            razoes.push(tb / tc);
            tb0 = tb0.min(tb);
            tc0 = tc0.min(tc);
            ib = b;
            ic = c;
        }
        razoes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |f: f64| razoes[((razoes.len() - 1) as f64 * f).round() as usize];
        let (mut pior, mut soma) = (0i32, 0i64);
        for (a, b) in ib.iter().zip(ic.iter()) {
            let d = (i32::from(*a) - i32::from(*b)).abs();
            pior = pior.max(d);
            soma += i64::from(d);
        }
        #[allow(clippy::cast_precision_loss)]
        let media = soma as f64 / ib.len() as f64;
        eprintln!(
            "  {raio:4.0} | {tb0:8.2} | {tc0:7.2} |   x{:5.2} ({:.2}..{:.2}) | {pior:9} | {media:6.3}",
            q(0.5),
            q(0.1),
            q(0.9)
        );
    }
}
